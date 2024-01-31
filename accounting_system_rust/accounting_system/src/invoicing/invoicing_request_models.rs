use anyhow::{Context, ensure};
use chrono::NaiveDate;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use invoice_doc_generator::hsc_sac::GstItemCode;
use invoice_doc_generator::invoice_line::line_number::LineNumber;
use invoice_doc_generator::invoice_line::line_quantity::LineQuantity;
use invoice_doc_generator::invoice_line::line_subtitle::LineSubtitle;
use invoice_doc_generator::invoice_line::line_title::LineTitle;
use invoice_doc_generator::invoice_line::unit_price::Price;
use invoice_doc_generator::percentages::tax_discount_cess::{CessPercentage, DiscountPercentage, GSTPercentage};

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateInvoiceRequest {
    pub idempotence_key: Uuid,
    pub tenant_id: Uuid,
    pub invoicing_series_mst_id: Uuid,
    pub currency_id: Uuid,
    pub service_invoice: bool,
    pub supplier_id: Uuid,
    //if its not registered, go register first
    pub billed_to_customer_id: Option<Uuid>,
    //there is id for generic customer too, go use that
    pub shipped_to_customer_id: Option<Uuid>,
    //there is id for generic customer too, go use that
    pub order_number: Option<PurchaseOrderNo>,
    pub order_date: Option<PurchaseOrderDate>,
    pub payment_terms: Option<PaymentTermsValidated>,
    pub invoice_lines: Vec<CreateInvoiceLineRequest>,
    pub additional_charges: Vec<CreateAdditionalChargeRequest>,
}

#[derive(Debug, Serialize, Deserialize, Builder, Clone)]
pub struct CreateAdditionalChargeRequest {
    line_no: Option<LineNumber>,
    line_title: LineTitle,
    rate: Price,
}

#[derive(Debug, Serialize, Deserialize, Builder, Clone)]
pub struct CreateInvoiceLineRequest {
    line_no: Option<LineNumber>,
    #[serde(flatten)]
    gst_item_code: GstItemCode,
    line_title: LineTitle,
    line_subtitle: Option<LineSubtitle>,
    quantity: LineQuantity,
    unit_price: Price,
    tax_rate_percentage: GSTPercentage,
    discount_percentage: DiscountPercentage,
    cess_percentage: CessPercentage,
    mrp: Option<Price>,
    batch_no: Option<BatchNo>,
    expiry_date: Option<ExpiryDateMs>,
}


#[derive(Debug, Serialize, Deserialize)]
struct PaymentTerms {
    due_days: DueDays,
    discount_days: Option<DiscountDays>,
    discount_percent: Option<DiscountPercentage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "PaymentTerms")]
pub struct PaymentTermsValidated {
    due_days: DueDays,
    discount_days: Option<DiscountDays>,
    discount_percent: Option<DiscountPercentage>,
}

impl TryFrom<PaymentTerms> for PaymentTermsValidated {
    type Error = anyhow::Error;

    fn try_from(value: PaymentTerms) -> Result<Self, Self::Error> {
        if let Some(discount_days) = &value.discount_days {
            ensure!(value.due_days.0 >= discount_days.0,
                "discount days {} cannot be more than due days {} in payment terms",
                discount_days.0,value.due_days.0);
        }
        Ok(PaymentTermsValidated {
            due_days: value.due_days,
            discount_days: value.discount_days,
            discount_percent: value.discount_percent,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "i32")]
pub struct DiscountDays(u16);

impl DiscountDays {
    pub fn new(value: i32) -> anyhow::Result<Self> {
        ensure!(value>=0,"discount days cannot be less than 0");
        Ok(DiscountDays(value as u16))
    }
}

impl TryFrom<i32> for DiscountDays {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        DiscountDays::new(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "i32")]
pub struct DueDays(u16);

impl DueDays {
    pub fn new(value: i32) -> anyhow::Result<Self> {
        ensure!(value>=0,"due days cannot be less than 0");
        ensure!(value<=400,"due days cannot be more than 400");
        Ok(DueDays(value as u16))
    }
}

impl TryFrom<i32> for DueDays {
    type Error = anyhow::Error;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        DueDays::new(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct PurchaseOrderNo(String);

impl PurchaseOrderNo {
    pub fn new(value: String) -> anyhow::Result<Self> {
        let value = value.trim();
        ensure!(value.len()<=30,"purchase order no cannot be more than 30 chars");
        ensure!(!value.is_empty(),"purchase order no cannot be empty, make it null if you don't need it");
        ensure!(value.chars()
            .all(|a| a.is_ascii_alphanumeric() || a == '/' || a == '-'),
            "purchase order no can only contain alphanumeric characters or / or -"
        );
        Ok(PurchaseOrderNo(value.to_owned()))
    }
}

impl TryFrom<String> for PurchaseOrderNo {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        PurchaseOrderNo::new(value)
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct PurchaseOrderDate(NaiveDate);

impl PurchaseOrderDate {
    pub fn new(value: String) -> anyhow::Result<Self> {
        let p = NaiveDate::parse_from_str(value.as_str(), "%Y-%m-%d").context("")?;
        Ok(PurchaseOrderDate(p))
    }
}

impl TryFrom<String> for PurchaseOrderDate {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        PurchaseOrderDate::new(value)
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct BatchNo(String);


impl BatchNo {
    pub fn new(value: String) -> anyhow::Result<Self> {
        let value = value.trim();
        ensure!(value.len()<=20,"batch no cannot be more than 20 chars but was {} chars",value.len());
        ensure!(value.chars().all(|a|a.is_ascii_alphanumeric()),"batch no can only contain alphanumeric characters");
        Ok(Self(value.to_owned()))
    }
}

impl TryFrom<String> for BatchNo {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        BatchNo::new(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct ExpiryDateMs(NaiveDate);

impl ExpiryDateMs {
    pub fn new(value: String) -> anyhow::Result<Self> {
        let p = NaiveDate::parse_from_str(value.as_str(), "%Y-%m-%d").context("")?;
        Ok(ExpiryDateMs(p))
    }
}

impl TryFrom<String> for ExpiryDateMs {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ExpiryDateMs::new(value)
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use lazy_static::lazy_static;
    use uuid::Uuid;

    use invoice_doc_generator::hsc_sac::{GstItemCode, Hsn};
    use invoice_doc_generator::invoice_line::line_number::LineNumber;
    use invoice_doc_generator::invoice_line::line_quantity::test_utils::a_line_quantity;
    use invoice_doc_generator::invoice_line::line_title::LineTitle;
    use invoice_doc_generator::invoice_line::unit_price::Price;
    use invoice_doc_generator::percentages::tax_discount_cess::{CessPercentage, DiscountPercentage, GSTPercentage};

    use crate::accounting::currency::currency_models::SEED_CURRENCY_ID;
    use crate::invoicing::invoicing_request_models::{CreateAdditionalChargeRequest, CreateAdditionalChargeRequestBuilder, CreateInvoiceLineRequest, CreateInvoiceLineRequestBuilder, CreateInvoiceRequest, CreateInvoiceRequestBuilder};
    use crate::invoicing::invoicing_series::invoicing_series_models::tests::SEED_INVOICING_SERIES_MST_ID;
    use crate::masters::business_entity_master::business_entity_models::tests::{SEED_BUSINESS_ENTITY_ID2, SEED_BUSINESS_ENTITY_INVOICE_DTL_ID1};
    use crate::tenant::tenant_models::SEED_TENANT_ID;

    lazy_static! {
        pub static ref SEED_INVOICE_ID:Uuid = Uuid::from_str("018d5559-745a-7371-80c6-a4efaa2cafe6").unwrap();
    }

    fn a_create_invoice_request(builder: CreateInvoiceRequestBuilder) -> CreateInvoiceRequest {
        CreateInvoiceRequest {
            idempotence_key: builder.idempotence_key.unwrap_or_else(Uuid::now_v7),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            invoicing_series_mst_id: builder.invoicing_series_mst_id.unwrap_or(*SEED_INVOICING_SERIES_MST_ID),
            currency_id: builder.currency_id.unwrap_or(*SEED_CURRENCY_ID),
            service_invoice: builder.service_invoice.unwrap_or(false),
            supplier_id: builder.supplier_id.unwrap_or(*SEED_BUSINESS_ENTITY_INVOICE_DTL_ID1),
            billed_to_customer_id: builder.billed_to_customer_id.unwrap_or(Some(*SEED_BUSINESS_ENTITY_ID2)),
            shipped_to_customer_id: builder.shipped_to_customer_id.unwrap_or(Some(*SEED_BUSINESS_ENTITY_ID2)),
            order_number: builder.order_number.unwrap(),
            order_date: builder.order_date.unwrap(),
            payment_terms: builder.payment_terms.unwrap(),
            invoice_lines: builder.invoice_lines
                .unwrap_or_else(||
                    vec![a_create_invoice_line_request(Default::default())]),
            additional_charges: builder.additional_charges
                .unwrap_or_else(|| vec![a_create_additional_charge_request(Default::default())]),

        }
    }

    fn a_create_additional_charge_request(builder: CreateAdditionalChargeRequestBuilder) -> CreateAdditionalChargeRequest {
        CreateAdditionalChargeRequest {
            line_no: builder.line_no.unwrap_or(Some(LineNumber::new(1).unwrap())),
            line_title: builder.line_title.unwrap_or(LineTitle::new("some line title".to_string()).unwrap()),
            rate: builder.rate.unwrap_or_else(|| Price::new(0.0).unwrap()),
        }
    }

    fn a_create_invoice_line_request(builder: CreateInvoiceLineRequestBuilder) -> CreateInvoiceLineRequest {
        CreateInvoiceLineRequest {
            line_no: builder.line_no.unwrap_or(Some(LineNumber::new(1).unwrap())),
            gst_item_code: builder.gst_item_code.unwrap_or(GstItemCode::HsnCode(Hsn::new("38220011".to_string()).unwrap())),
            line_title: builder.line_title.unwrap_or(LineTitle::new("some random line title".to_string()).unwrap()),
            line_subtitle: builder.line_subtitle.unwrap(),
            quantity: builder.quantity.unwrap_or_else(|| a_line_quantity(Default::default())),
            unit_price: builder.unit_price.unwrap_or_else(|| Price::new(10.0).unwrap()),
            tax_rate_percentage: builder.tax_rate_percentage.unwrap_or_else(|| GSTPercentage::new(28).unwrap()),
            discount_percentage: builder.discount_percentage.unwrap_or_else(|| DiscountPercentage::new(0.0).unwrap()),
            cess_percentage: builder.cess_percentage.unwrap_or_else(|| CessPercentage::new(0.0).unwrap()),
            mrp: builder.mrp.unwrap(),
            batch_no: builder.batch_no.unwrap(),
            expiry_date: builder.expiry_date.unwrap(),
        }
    }
}