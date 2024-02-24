use std::cmp::Ordering;
use std::error::Error;
use std::future::Future;
use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use invoice_doc_generator::hsc_sac::GstItemCode::{HsnCode, SacCode};

use crate::accounting::currency::currency_service::CurrencyService;
use crate::common_utils::dao_error::DaoError;
use crate::common_utils::utils::current_indian_date;
use crate::invoicing::invoice_template::invoice_template_service::InvoiceTemplateService;
use crate::invoicing::invoicing_dao::InvoicingDao;
use crate::invoicing::invoicing_dao_models::convert_to_invoice_db;
use crate::invoicing::invoicing_request_models::CreateInvoiceRequest;
use crate::invoicing::invoicing_series::invoicing_series_service::InvoicingSeriesService;
use crate::masters::business_entity_master::business_entity_service::BusinessEntityService;
use crate::masters::company_master::company_master_models::gstin_no::GstinNo;
use crate::tenant::tenant_service::TenantService;

#[derive(Debug, Error)]
pub enum InvoicingServiceError {
    #[error("error in db {}", 0)]
    Db(#[from]DaoError),
    #[error("validation failures \n {}", .0.join("\n"))]
    Validation(Vec<String>),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

#[async_trait]
pub trait InvoicingService {
    async fn create_invoice(&self, req: &CreateInvoiceRequest, tenant_id: Uuid, user_id: Uuid) -> Result<Uuid, InvoicingServiceError>;
}

#[allow(dead_code)]
struct InvoicingServiceImpl {
    dao: Arc<dyn InvoicingDao>,
    tenant_service: Arc<dyn TenantService>,
    currency_service: Arc<dyn CurrencyService>,
    invoicing_series_service: Arc<dyn InvoicingSeriesService>,
    business_entity_service: Arc<dyn BusinessEntityService>,
    invoice_template_service: Arc<dyn InvoiceTemplateService>,
}

impl InvoicingServiceImpl {
    async fn validate_create_invoice_request(&self, req: &CreateInvoiceRequest, tenant_id: Uuid) -> Result<(), InvoicingServiceError> {
        let mut errors: Vec<String> = vec![];
        Self::validate_invoice_lines(req, &mut errors);
        Self::validate_order_date(req, &mut errors);
        Self::validate_invoice_bill_ship_detail(req, &mut errors);
        Self::validate_invoice_lines_gst_codes(req, &mut errors);
        self.validate_ids(req, tenant_id, &mut errors).await?;
        if !errors.is_empty() {
            Err(InvoicingServiceError::Validation(errors))
        } else {
            Ok(())
        }
    }
    fn validate_invoice_lines_gst_codes(req: &CreateInvoiceRequest, errors: &mut Vec<String>) {
        req.invoice_lines.iter().for_each(|a| {
            match a.gst_item_code {
                HsnCode(_) => {
                    if req.service_invoice {
                        let p =
                            format!("hsn code found for service invoice.Use sac code instead.Line title: {}",
                                    a.line_title.inner());
                        errors.push(p);
                    }
                }
                SacCode(_) => {
                    if !req.service_invoice {
                        let p =
                            format!("sac code found for non service invoice.Use hsn code instead.Line title: {}",
                                    a.line_title.inner());
                        errors.push(p);
                    }
                }
            };
        });
    }
    fn validate_invoice_bill_ship_detail(req: &CreateInvoiceRequest, errors: &mut Vec<String>) {
        if req.b2b_invoice && req.bill_ship_detail.is_none() {
            errors.push("bill_ship_detail is mandatory if b2b_invoice".to_string())
        }
        if let Some(bill_ship_detail) = req.bill_ship_detail.as_ref() {
            if req.supplier_id == bill_ship_detail.billed_to_customer_id {
                errors.push("supplier id and billed_to_customer_id cannot be same".to_string());
            }
            if req.supplier_id == bill_ship_detail.shipped_to_customer_id {
                errors.push("supplier id and shipped_to_customer_id cannot be same".to_string());
            }
        }
    }
    fn validate_invoice_lines(req: &CreateInvoiceRequest, errors: &mut Vec<String>) {
        if req.invoice_lines.is_empty() {
            errors.push("atleast one invoice line is required".to_string());
        }
    }
    fn validate_order_date(req: &CreateInvoiceRequest, errors: &mut Vec<String>) {
        if let Some(date) = req.order_date.as_ref() {
            let curr_date = current_indian_date();
            if date.get_date().cmp(&curr_date) == Ordering::Greater {
                errors.push("purchase order date cannot be of future while generating invoice".to_string())
            }
        }
    }

    async fn validate_ids(&self, req: &CreateInvoiceRequest, tenant_id: Uuid,
                          errors: &mut Vec<String>)
                          -> Result<(), InvoicingServiceError>
    {
        async fn wrap<T: Error + Send + Sync + 'static>(fetch_block: impl Future<Output=Result<bool, T>>, err_msg: &str, errors: &mut Vec<String>) -> anyhow::Result<()> {
            let valid = fetch_block.await.context("error during id validation")?;
            if !valid {
                errors.push(err_msg.to_string())
            }
            Ok(())
        }

        wrap(async {
            self.currency_service
                .get_currency_entry(req.currency_id, tenant_id).await
                .map(|a| a.is_some())
        }, "currency id does not exists for this tenant id", errors).await?;
        wrap(
            self.business_entity_service
                .is_valid_business_entity_id(&req.supplier_id, &tenant_id)
            , "supplier id does not exists for this tenant id", errors).await?;

        if let Some(bill_ship) = req.bill_ship_detail.as_ref() {
            wrap(self.business_entity_service
                     .is_valid_business_entity_id(&bill_ship.billed_to_customer_id, &tenant_id),
                 "bill_to_id does not exists for this tenant id", errors).await?;
            wrap(
                self.business_entity_service
                    .is_valid_business_entity_id(&bill_ship.shipped_to_customer_id, &tenant_id),
                "ship_to_id does not exists for this tenant id", errors,
            ).await?;
        }
        wrap(
            self.invoice_template_service
                .is_valid_template_id(req.invoice_template_id, tenant_id),
            "invoice template id does not exists for this tenant id", errors,
        ).await?;
        Ok(())
    }

    async fn igst_applicable(&self, supplier_id: Uuid, bill_to_id: Option<Uuid>, tenant_id: Uuid)
                             -> anyhow::Result<bool> {
        return if let Some(bill_to_id) = bill_to_id {
            let supplier =
                self.business_entity_service
                    .get_business_entity_by_id(&supplier_id, &tenant_id).await
                    .context("supplier fetch get_business_entity_by_id failed")?
                    .with_context(|| format!("business entity id not found for id:{}", supplier_id))?;
            
            let bill_to = self.business_entity_service
                .get_business_entity_by_id(&bill_to_id, &tenant_id).await
                .context("bill to fetch get_business_entity_by_id failed")?
                .with_context(|| format!("business entity id not found for id:{}", bill_to_id))?;
            let supplier_gstin = supplier.business_entity.entity_type.extract_gstin()
                .with_context(|| format!("could not extract gstin from supplier id {}", 
                                         supplier.business_entity
                    .base_master_fields.id))?;
            let bill_to_gstin = bill_to.business_entity.entity_type.extract_gstin();
            if let Some(bill_to_gstin) = bill_to_gstin {
                is_gstin_from_same_state(supplier_gstin, bill_to_gstin)
                    .map(|a| !a)
            } else {
                return Ok(false);
            }
        } else {
            Ok(false)
        };
    }
}

fn is_gstin_from_same_state(gstin1: &GstinNo, gstin2: &GstinNo) -> anyhow::Result<bool> {
    Ok(gstin1.get_str().get(0..2)
        .context("gstin short than 2")? ==
        gstin2.get_str().get(0..2)
            .context("gstin short than 2")?)
}

#[async_trait]
impl InvoicingService for InvoicingServiceImpl {
    async fn create_invoice(&self, req: &CreateInvoiceRequest, tenant_id: Uuid, user_id: Uuid) -> Result<Uuid, InvoicingServiceError> {
        self.validate_create_invoice_request(req, tenant_id).await?;
        let curr = self.currency_service
            .get_currency_entry(req.currency_id, tenant_id).await
            .context("err while fetching currency from db")?
            .context("currency not found in db")?;
        let igst_applicable = self.igst_applicable(req.supplier_id,
                                                   req.bill_ship_detail.as_ref()
                                                       .map(|a| a.billed_to_customer_id),
                                                   tenant_id).await?;
        let db_model = convert_to_invoice_db(req, curr.scale, igst_applicable, user_id, tenant_id)?;
        self.dao.create_invoice(&db_model).await?;
        todo!()
        // req.
        //validate
        //calculate invoice fields
    }

    //template_id,series_mst_id,currency_id,supplier_id,billed_to,shipped_to ids must exist for this tenant
}


#[cfg(test)]
mod tests {
    use chrono::Days;
    use spectral::assert_that;
    use spectral::prelude::VecAssertions;

    use invoice_doc_generator::hsc_sac::{Hsn, Sac};
    use invoice_doc_generator::hsc_sac::GstItemCode::{HsnCode, SacCode};

    use crate::common_utils::utils::current_indian_date;
    use crate::invoicing::invoicing_request_models::PurchaseOrderDate;
    use crate::invoicing::invoicing_request_models::tests::{a_create_invoice_line_request, a_create_invoice_request};
    use crate::invoicing::invoicing_service::InvoicingServiceImpl;

    #[tokio::test]
    async fn test_validate_order_date() {
        let mut req = a_create_invoice_request(Default::default());
        let future_date = current_indian_date().checked_add_days(Days::new(3)).unwrap();
        req.order_date = Some(PurchaseOrderDate::from_date(future_date).unwrap());
        let mut errors: Vec<String> = vec![];

        InvoicingServiceImpl::validate_order_date(&req, &mut errors);
        let past_date = future_date.checked_sub_days(Days::new(3)).unwrap();
        req.order_date = Some(PurchaseOrderDate::from_date(past_date).unwrap());
        let mut errors2: Vec<String> = vec![];

        InvoicingServiceImpl::validate_order_date(&req, &mut errors2);
        assert_that!(errors).has_length(1);
        assert_that!(errors[0])
            .is_equal_to("purchase order date cannot be of future while generating invoice".to_string());
        assert_that!(errors2).is_empty();
    }

    #[tokio::test]
    async fn test_validate_invoice_lines() {
        let mut req = a_create_invoice_request(Default::default());
        req.invoice_lines = vec![];
        let mut errors: Vec<String> = vec![];
        InvoicingServiceImpl::validate_invoice_lines(&req, &mut errors);
        assert_that!(errors).has_length(1);
        assert_that!(errors[0]).
            is_equal_to("atleast one invoice line is required".to_string());
        let req = a_create_invoice_request(Default::default());
        let mut errors: Vec<String> = vec![];
        InvoicingServiceImpl::validate_invoice_lines(&req, &mut errors);
        assert_that!(errors).is_empty();
    }

    #[tokio::test]
    async fn test_validate_invoice_lines_gst_codes() {
        let mut req = a_create_invoice_request(Default::default());
        let mut line = a_create_invoice_line_request(Default::default());
        line.gst_item_code = HsnCode(Hsn::new("01011090".to_string()).unwrap());
        req.invoice_lines = vec![line];
        req.service_invoice = true;
        let mut errors: Vec<String> = vec![];
        InvoicingServiceImpl::validate_invoice_lines_gst_codes(&req, &mut errors);
        assert_that!(errors).has_length(1);
        assert_that!(errors[0])
            .is_equal_to(
                "hsn code found for service invoice.Use sac code instead.Line title: some random line title"
                    .to_string());
        req.service_invoice = false;
        let mut errors: Vec<String> = vec![];
        InvoicingServiceImpl::validate_invoice_lines_gst_codes(&req, &mut errors);
        assert_that!(errors).is_empty();
        let mut line = a_create_invoice_line_request(Default::default());
        line.gst_item_code = SacCode(Sac::new("995425".to_string()).unwrap());
        req.invoice_lines = vec![line];
        req.service_invoice = true;
        errors = vec![];
        InvoicingServiceImpl::validate_invoice_lines_gst_codes(&req, &mut errors);
        assert_that!(errors).is_empty();
        req.service_invoice = false;
        errors = vec![];
        InvoicingServiceImpl::validate_invoice_lines_gst_codes(&req, &mut errors);
        assert_that!(errors).has_length(1);
        assert_that!(errors[0]).is_equal_to("sac code found for non service invoice.Use hsn code instead.Line title: some random line title".to_string())
    }

    #[tokio::test]
    async fn test_validate_bill_ship_detail() {
        let mut req = a_create_invoice_request(Default::default());
        req.bill_ship_detail.as_mut().unwrap().billed_to_customer_id = req.supplier_id;
        let mut errors: Vec<String> = vec![];
        InvoicingServiceImpl::validate_invoice_bill_ship_detail(&req, &mut errors);
        assert_that!(errors).has_length(1);
        assert_that!(errors[0]).is_equal_to("supplier id and billed_to_customer_id cannot be same".to_string());
        let mut req = a_create_invoice_request(Default::default());
        req.bill_ship_detail.as_mut().unwrap().shipped_to_customer_id = req.supplier_id;
        let mut errors: Vec<String> = vec![];
        InvoicingServiceImpl::validate_invoice_bill_ship_detail(&req, &mut errors);
        assert_that!(errors).has_length(1);
        assert_that!(errors[0]).is_equal_to("supplier id and shipped_to_customer_id cannot be same".to_string());
    }
}