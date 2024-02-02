use std::cmp::Ordering;

use async_trait::async_trait;

use invoice_doc_generator::hsc_sac::GstItemCode::{HsnCode, SacCode};

use crate::common_utils::utils::current_indian_date;
use crate::invoicing::invoicing_request_models::CreateInvoiceRequest;

#[async_trait]
pub trait InvoicingService {
    fn create_invoice(req: &CreateInvoiceRequest);
}


struct InvoicingServiceImpl {}

impl InvoicingServiceImpl {
    fn validate_create_invoice_request(req: &CreateInvoiceRequest) -> Result<(), Vec<String>> {
        let mut errors: Vec<String> = vec![];
        InvoicingServiceImpl::validate_invoice_lines(req, &mut errors);
        InvoicingServiceImpl::validate_order_date(req, &mut errors);
        InvoicingServiceImpl::validate_invoice_bill_ship_detail(req, &mut errors);
        InvoicingServiceImpl::validate_invoice_lines_gst_codes(req, &mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
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
}

impl InvoicingService for InvoicingServiceImpl {
    fn create_invoice(req: &CreateInvoiceRequest) {
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
        let  req = a_create_invoice_request(Default::default());
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
        errors=vec![];
        InvoicingServiceImpl::validate_invoice_lines_gst_codes(&req, &mut errors);
        assert_that!(errors).is_empty();
        req.service_invoice = false;
        errors=vec![];
        InvoicingServiceImpl::validate_invoice_lines_gst_codes(&req, &mut errors);
        assert_that!(errors).has_length(1);
        assert_that!(errors[0]).is_equal_to("sac code found for non service invoice.Use hsn code instead.Line title: some random line title".to_string())
    }
    #[tokio::test]
    async fn test_validate_bill_ship_detail(){
        let mut req = a_create_invoice_request(Default::default());
        req.bill_ship_detail.as_mut().unwrap().billed_to_customer_id=req.supplier_id;
        let mut errors:Vec<String> = vec![];
        InvoicingServiceImpl::validate_invoice_bill_ship_detail(&req, &mut errors);
        assert_that!(errors).has_length(1);
        assert_that!(errors[0]).is_equal_to("supplier id and billed_to_customer_id cannot be same".to_string());
        let mut req = a_create_invoice_request(Default::default());
        req.bill_ship_detail.as_mut().unwrap().shipped_to_customer_id=req.supplier_id;
        let mut errors:Vec<String> = vec![];
        InvoicingServiceImpl::validate_invoice_bill_ship_detail(&req, &mut errors);
        assert_that!(errors).has_length(1);
        assert_that!(errors[0]).is_equal_to("supplier id and shipped_to_customer_id cannot be same".to_string());

    }
}