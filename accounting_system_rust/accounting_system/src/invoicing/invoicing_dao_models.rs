use uuid::Uuid;
use xxhash_rust::xxh32;

use crate::invoicing::invoicing_request_models::{CreateAdditionalChargeRequest, PaymentTermsValidated};

#[derive(Debug)]
pub struct PaymentTermsDb {
    pub due_days: i32,
    pub discount_days: Option<i32>,
    pub discount_percent: Option<f32>,
}

#[derive(Debug)]
pub struct InvoiceLineDb<'a> {
    pub line_id: Uuid,
    pub line_no: &'a str,
    pub hsn_sac_code: &'a str,
    pub line_title: &'a str,
    pub line_title_sac_hash: i64,
    pub line_subtitle: &'a str,
    pub subtitle_hash: i64,
    pub quantity: f64,
    pub uqc: &'a str,
    pub unit_price: f64,
    pub tax_rate_percentage: f32,
    pub discount_percentage: f32,
    pub cess_percentage: f32,
    pub mrp: Option<f64>,
    pub batch_no: Option<&'a str>,
    pub expiry_date_ms: Option<i64>,
    pub line_net_total: f64,
    pub igst_applicable: bool,
}

#[derive(Debug)]
pub struct AdditionalChargeDb<'a> {
    pub line_id: Uuid,
    pub line_no: i16,
    pub line_title: &'a str,
    pub line_title_xx_hash: i64,
    pub rate: f64,
}

#[derive(Debug)]
pub struct InvoiceDb<'a> {
    pub idempotence_key: Uuid,
    pub tenant_id: Uuid,
    pub invoice_template_id: Uuid,
    pub invoicing_series_mst_id: Uuid,
    pub invoice_date_ms: i64,
    pub currency_id: Uuid,
    pub service_invoice: bool,
    pub b2b_invoice: bool,
    pub e_invoice_applicable: bool,
    pub supplier_id: Uuid,
    pub billed_to_customer_id: Option<Uuid>,
    pub shipped_to_customer_id: Option<Uuid>,
    pub order_number: Option<&'a str>,
    pub order_date: Option<i64>,
    pub payment_terms: Option<PaymentTermsDb>,
    pub invoice_lines: Vec<InvoiceLineDb<'a>>,
    pub additional_charges: Vec<AdditionalChargeDb<'a>>,
    pub financial_year: i16,
    pub total_taxable_amount: f64,
    pub total_tax_amount: f64,
    pub total_additional_charges_amount: f64,
    pub round_off: f64,
    pub total_payable_amount: f64,
    pub created_by: Uuid,
}


fn convert_to_payment_terms_db(req: &PaymentTermsValidated) -> PaymentTermsDb {
    PaymentTermsDb {
        due_days: req.due_days.inner() as i32,
        discount_days: req.discount_days.as_ref().map(|a| a.inner() as i32),
        discount_percent: req.discount_percent.as_ref()
            .map(|a| a.inner())
    }
}

fn convert_to_additional_charge_db(req: &CreateAdditionalChargeRequest, line_no: i16) -> AdditionalChargeDb {
    AdditionalChargeDb {
        line_id: Uuid::now_v7(),
        line_no,
        line_title: req.line_title.inner(),
        line_title_xx_hash: compute_32_bit_xx_hash(req.line_title.inner()),
        rate: req.rate.inner(),
    }
}

fn compute_32_bit_xx_hash(st: &str) -> i64 {
    let mut hasher = xxh32::Xxh32::new(0);
    hasher.update(st.as_bytes());
    hasher.digest() as i64
}