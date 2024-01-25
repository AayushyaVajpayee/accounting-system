use anyhow::{Context, ensure};
use chrono::NaiveDate;
use invoice_doc_generator::hsc_sac::GstItemCode;
use invoice_doc_generator::invoice_line::line_number::LineNumber;
use invoice_doc_generator::invoice_line::line_quantity::LineQuantity;
use invoice_doc_generator::invoice_line::line_subtitle::LineSubtitle;
use invoice_doc_generator::invoice_line::line_title::LineTitle;
use invoice_doc_generator::invoice_line::unit_price::Price;
use invoice_doc_generator::invoice_number::InvoiceNumber;
use invoice_doc_generator::percentages::tax_discount_cess::{CessPercentage, DiscountPercentage, GSTPercentage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvoiceRequest {
    pub idempotence_key: Uuid,
    pub tenant_id: Uuid,
    pub invoice_number: InvoiceNumber,
    pub currency_id: Uuid,
    pub service_invoice: bool,
    pub supplier_id: Uuid,
    //if its not registered, go register first
    pub billed_to_customer_id: Uuid,
    //there is id for generic customer too, go use that
    pub shipped_to_customer_id: Uuid,
    //there is id for generic customer too, go use that
    pub order_number: Option<PurchaseOrderNo>,
    pub order_date: Option<PurchaseOrderDate>,
    pub payment_terms: PaymentTermsValidated,
    pub invoice_lines: Vec<CreateInvoiceLineRequest>,
    pub additional_charges: Vec<CreateAdditionalChargeRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAdditionalChargeRequest {
    line_no: Option<LineNumber>,
    line_title: LineTitle,
    rate: Price,
}

#[derive(Debug, Serialize, Deserialize)]
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
    mrp: Price,
    batch_no: BatchNo,
    expiry_date: ExpiryDateMs,
}


#[derive(Debug, Serialize, Deserialize)]
struct PaymentTerms {
    due_days: DueDays,
    discount_days: Option<DiscountDays>,
    discount_percent: Option<DiscountPercentage>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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


#[derive(Debug, Serialize, Deserialize)]
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


#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

