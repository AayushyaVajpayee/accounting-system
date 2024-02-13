use anyhow::anyhow;
use chrono::TimeZone;
use itertools::Itertools;
use uuid::Uuid;
use xxhash_rust::xxh32;

use crate::common_utils::utils::current_indian_financial_year;
use crate::invoicing::invoicing_request_models::{CreateAdditionalChargeRequest, CreateInvoiceLineRequest, CreateInvoiceRequest, PaymentTermsValidated};

#[derive(Debug)]
pub struct PaymentTermsDb {
    pub due_days: i32,
    pub discount_days: Option<i32>,
    pub discount_percent: Option<f32>,
}

#[derive(Debug)]
pub struct InvoiceLineDb<'a> {
    pub line_id: Uuid,
    pub line_no: i16,
    pub hsn_sac_code: &'a str,
    pub line_title: &'a str,
    pub line_title_sac_hash: i64,
    pub line_subtitle: Option<&'a str>,
    pub subtitle_hash: Option<i64>,
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
    pub igst_applicable:bool
}


fn convert_to_payment_terms_db(req: &PaymentTermsValidated) -> PaymentTermsDb {
    PaymentTermsDb {
        due_days: req.due_days.inner() as i32,
        discount_days: req.discount_days.as_ref().map(|a| a.inner() as i32),
        discount_percent: req.discount_percent.as_ref()
            .map(|a| a.inner()),
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

fn convert_to_invoice_line_db(req: &CreateInvoiceLineRequest, line_no: i16) -> anyhow::Result<InvoiceLineDb> {
    let mut hasher = xxh32::Xxh32::new(0);
    hasher.update(req.line_title.inner().as_bytes());
    hasher.update(req.gst_item_code.as_str().as_bytes());
    let hash = hasher.digest() as i64;
    Ok(InvoiceLineDb {
        line_id: Uuid::now_v7(),
        line_no,
        hsn_sac_code: req.gst_item_code.as_str(),
        line_title: req.line_title.inner(),
        line_title_sac_hash: hash,
        line_subtitle: req.line_subtitle.as_ref().map(|a| a.inner()),
        subtitle_hash: req.line_subtitle.as_ref()
            .map(|a| compute_32_bit_xx_hash(a.inner())),
        quantity: req.quantity.get_quantity(),
        uqc: req.quantity.uom_as_str(),
        unit_price: req.unit_price.inner(),
        tax_rate_percentage: req.tax_rate_percentage.inner(),
        discount_percentage: req.discount_percentage.inner(),
        cess_percentage: req.cess_percentage.inner(),
        mrp: req.mrp.as_ref().map(|a| a.inner()),
        batch_no: req.batch_no.as_ref().map(|a| a.inner()),
        expiry_date_ms: req.expiry_date.as_ref().map(|a| a.epoch_millis()).flatten(),
        line_net_total:req.net_line_total()?,
    })
}

fn convert_to_invoice_db(req:&CreateInvoiceRequest,currency_scale:u32,igst_applicable:bool,created_by:Uuid)->anyhow::Result<InvoiceDb>{
    let date = chrono::Utc::now().naive_utc();
    Ok(InvoiceDb{
        idempotence_key: req.idempotence_key,
        tenant_id: req.tenant_id,
        invoice_template_id: req.invoice_template_id,
        invoicing_series_mst_id: req.invoicing_series_mst_id,
        invoice_date_ms: chrono_tz::Asia::Kolkata.from_utc_datetime(&date)
            .date_naive()
            .and_hms_opt(0,0,0)
            .map(|a|a.timestamp_millis())
            .ok_or_else(||anyhow!("error during invoice date computation"))?,
        currency_id: req.currency_id,
        service_invoice: req.service_invoice,
        b2b_invoice: req.b2b_invoice,
        e_invoice_applicable: req.einvoicing_applicable,
        supplier_id: req.supplier_id,
        billed_to_customer_id: req.bill_ship_detail
            .as_ref()
            .map(|l|l.billed_to_customer_id),
        shipped_to_customer_id: req.bill_ship_detail
            .as_ref()
            .map(|l|l.shipped_to_customer_id),
        order_number: req.order_number.as_ref()
            .map(|a|a.inner()),
        order_date: req.order_date.as_ref()
            .map(|a|a.epoch_millis()).flatten(),
        payment_terms: req.payment_terms.as_ref()
            .map(|a|convert_to_payment_terms_db(&a)),
        invoice_lines:  req.invoice_lines
            .iter()
            .enumerate()
            .map(|a|convert_to_invoice_line_db(a.1,a.0 as i16))
            .collect::<anyhow::Result<Vec<InvoiceLineDb>>>()?,
        additional_charges:req.additional_charges
            .iter()
            .enumerate()
            .map(|a|convert_to_additional_charge_db(a.1,a.0 as i16))
            .collect_vec(),
        financial_year: current_indian_financial_year() as i16,
        total_taxable_amount: req.total_taxable_amount()?,
        total_tax_amount: req.total_tax_amount()?,
        total_additional_charges_amount: req.total_additional_charge_amount(),
        round_off: 0.0,
        total_payable_amount: req.total_amount(currency_scale)?,
        created_by,
        igst_applicable,
    })
}

fn compute_32_bit_xx_hash(st: &str) -> i64 {
    let mut hasher = xxh32::Xxh32::new(0);
    hasher.update(st.as_bytes());
    hasher.digest() as i64
}