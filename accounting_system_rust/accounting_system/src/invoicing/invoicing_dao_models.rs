use anyhow::{anyhow, Context};
use chrono::TimeZone;
use itertools::Itertools;
use tokio_postgres::types::ToSql;
use uuid::Uuid;
use xxhash_rust::xxh32;

use crate::common_utils::pg_util::pg_util::{create_composite_type_db_row, ToPostgresString};
use crate::common_utils::utils::current_indian_financial_year;
use crate::invoicing::invoicing_request_models::{CreateAdditionalChargeRequest, CreateInvoiceLineRequestWithAllDetails, CreateInvoiceWithAllDetailsIncluded, PaymentTermsValidated};

#[derive(Debug, ToSql)]
#[postgres(name = "create_payment_terms_request")]
pub struct PaymentTermsDb {
    pub due_days: i32,
    pub discount_days: Option<i32>,
    pub discount_percent: Option<f32>,
}

impl ToPostgresString for PaymentTermsDb {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        let fields: &[&dyn ToPostgresString] = &[&self.due_days, &self.discount_days, &self.discount_percent];
        create_composite_type_db_row(fields, f)
    }

    fn db_type_name(&self) -> &'static str {
        "create_payment_terms_request"
    }
}

#[derive(Debug, ToSql)]
#[postgres(name = "create_invoice_line_request")]
pub struct InvoiceLineDb<'a> {
    pub line_id: Uuid,
    pub line_no: i16,
    pub hsn_sac_code: &'a str,
    pub line_title: &'a str,
    pub title_hsn_sac_hash: i64,
    pub line_subtitle: Option<&'a str>,
    pub subtitle_hash: Option<i64>,
    pub quantity: f64,
    pub free_quantity: f64,
    pub uqc: &'a str,
    pub unit_price: f64,
    pub tax_percentage: f32,
    pub discount_percentage: f32,
    pub cess_percentage: f32,
    pub cess_amount_per_unit: f64,
    pub retail_sale_price_for_cess: f64,
    pub cess_calculation_strategy: &'static str,
    pub mrp: Option<f32>,
    pub batch_no: Option<&'a str>,
    pub expiry_date_ms: Option<i64>,
    pub line_net_total: f64,
    pub reverse_charge_applicable: bool,
}

impl ToPostgresString for InvoiceLineDb<'_> {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        let fields: &[&dyn ToPostgresString] = &[
            &self.line_id,
            &self.line_no,
            &self.hsn_sac_code,
            &self.line_title,
            &self.title_hsn_sac_hash,
            &self.line_subtitle,
            &self.subtitle_hash,
            &self.quantity,
            &self.free_quantity,
            &self.uqc,
            &self.unit_price,
            &self.tax_percentage,
            &self.discount_percentage,
            &self.cess_percentage,
            &self.cess_amount_per_unit,
            &self.retail_sale_price_for_cess,
            &self.cess_calculation_strategy,
            &self.mrp,
            &self.batch_no,
            &self.expiry_date_ms,
            &self.line_net_total,
            &self.reverse_charge_applicable
        ];
        create_composite_type_db_row(fields, f)
    }

    fn db_type_name(&self) -> &'static str {
        "create_invoice_line_request"
    }
}

#[derive(Debug, ToSql)]
#[postgres(name = "create_additional_charge_request")]
pub struct AdditionalChargeDb<'a> {
    pub line_id: Uuid,
    pub line_no: i16,
    pub line_title: &'a str,
    pub title_xx_hash: i64,
    pub rate: f64,
}

impl ToPostgresString for AdditionalChargeDb<'_> {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        let fields: &[&dyn ToPostgresString] = &[
            &self.line_id,
            &self.line_no,
            &self.line_title,
            &self.title_xx_hash,
            &self.rate,
        ];
        create_composite_type_db_row(fields, f)
    }

    fn db_type_name(&self) -> &'static str {
        "create_additional_charge_request"
    }
}

#[derive(Debug, ToSql)]
#[postgres(name = "create_invoice_request")]
pub struct InvoiceDb<'a> {
    pub idempotence_key: Uuid,
    pub tenant_id: Uuid,
    pub invoice_template_id: Uuid,
    pub invoicing_series_mst_id: Uuid,
    pub invoice_date_ms: i64,
    pub currency_id: Uuid,
    pub service_invoice: bool,
    pub b2b_invoice: bool,
    pub e_invoicing_applicable: bool,
    pub supplier_id: Uuid,
    pub dispatch_from_id: Uuid,
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
    pub igst_applicable: bool,
    pub invoice_remarks: Option<&'a str>,
    pub ecommerce_gstin: Option<&'a str>,
}

impl ToPostgresString for InvoiceDb<'_> {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        let fields: &[&dyn ToPostgresString] = &[
            &self.idempotence_key,
            &self.tenant_id,
            &self.invoice_template_id,
            &self.invoicing_series_mst_id,
            &self.invoice_date_ms,
            &self.currency_id,
            &self.service_invoice,
            &self.b2b_invoice,
            &self.e_invoicing_applicable,
            &self.supplier_id,
            &self.dispatch_from_id,
            &self.billed_to_customer_id,
            &self.shipped_to_customer_id,
            &self.order_number,
            &self.order_date,
            &self.payment_terms,
            &self.invoice_lines,
            &self.additional_charges,
            &self.financial_year,
            &self.total_taxable_amount,
            &self.total_tax_amount,
            &self.total_additional_charges_amount,
            &self.round_off,
            &self.total_payable_amount,
            &self.created_by,
            &self.igst_applicable,
            &self.invoice_remarks,
            &self.ecommerce_gstin,
        ];
        create_composite_type_db_row(fields, f)
    }

    fn db_type_name(&self) -> &'static str {
        "create_invoice_request"
    }
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
        title_xx_hash: compute_32_bit_xx_hash(req.line_title.inner()),
        rate: req.rate.inner(),
    }
}

fn convert_to_invoice_line_db(req: &CreateInvoiceLineRequestWithAllDetails, line_no: i16) -> anyhow::Result<InvoiceLineDb> {
    let mut hasher = xxh32::Xxh32::new(0);
    hasher.update(req.product_item_id.title.inner().as_bytes());
    hasher.update(req.product_item_id.hsn_sac_code.as_str().as_bytes());
    let hash = hasher.digest() as i64;
    let cess_strategy = req.product_item_id.get_cess_rate()?;
    Ok(InvoiceLineDb {
        line_id: Uuid::now_v7(),
        line_no,
        hsn_sac_code: req.product_item_id.hsn_sac_code.as_str(),
        line_title: req.product_item_id.title.inner(),
        title_hsn_sac_hash: hash,
        line_subtitle: req.product_item_id.subtitle.as_ref().map(|a| a.inner()),
        subtitle_hash: req.product_item_id.subtitle.as_ref()
            .map(|a| compute_32_bit_xx_hash(a.inner())),
        quantity: req.quantity.get_quantity(),
        uqc: req.quantity.uom_as_str(),
        unit_price: req.unit_price.inner(),
        tax_percentage: req.product_item_id.get_tax_rate()?.tax_rate_percentage.inner(),
        discount_percentage: req.discount_percentage.inner(),
        cess_percentage: cess_strategy.cess_strategy.get_cess_rate_percentage()
            .context("cess percentage cannot be none")?,
        cess_amount_per_unit: cess_strategy.cess_strategy
            .get_cess_amount_per_unit()
            .unwrap_or(0.0),
        retail_sale_price_for_cess: cess_strategy.cess_strategy
            .get_retail_sale_price()
            .unwrap_or(0.0),
        cess_calculation_strategy: cess_strategy.cess_strategy.get_strategy_name(),
        mrp: req.mrp.as_ref().map(|a| a.inner() as f32),
        batch_no: req.batch_no.as_ref().map(|a| a.inner()),
        expiry_date_ms: req.expiry_date.as_ref().map(|a| a.epoch_millis()).flatten(),
        line_net_total: req.net_line_total()?,
        reverse_charge_applicable: req.reverse_charge_applicable,
        free_quantity: req.free_quantity.get_quantity(),
    })
}

pub fn convert_to_invoice_db(req: &CreateInvoiceWithAllDetailsIncluded, currency_scale: i16, igst_applicable: bool,
                             created_by: Uuid, tenant_id: Uuid) -> anyhow::Result<InvoiceDb> {
    let date = chrono::Utc::now().naive_utc();
    Ok(InvoiceDb {
        idempotence_key: req.idempotence_key,
        tenant_id,
        invoice_template_id: req.invoice_template_id,
        invoicing_series_mst_id: req.invoicing_series_mst_id,
        invoice_date_ms: chrono_tz::Asia::Kolkata.from_utc_datetime(&date)
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .map(|a| a.timestamp_millis())
            .ok_or_else(|| anyhow!("error during invoice date computation"))?,
        currency_id: req.currency_id,
        service_invoice: req.service_invoice,
        b2b_invoice: req.b2b_invoice,
        e_invoicing_applicable: req.einvoicing_applicable,
        supplier_id: req.supplier_id,
        dispatch_from_id: req.dispatch_from_id.unwrap_or(req.supplier_id),
        billed_to_customer_id: req.bill_ship_detail
            .as_ref()
            .map(|l| l.billed_to_customer_id),
        shipped_to_customer_id: req.bill_ship_detail
            .as_ref()
            .map(|l| l.shipped_to_customer_id),
        order_number: req.order_number.as_ref()
            .map(|a| a.inner()),
        order_date: req.order_date.as_ref()
            .map(|a| a.epoch_millis()).flatten(),
        payment_terms: req.payment_terms.as_ref()
            .map(|a| convert_to_payment_terms_db(&a)),
        invoice_lines: req.invoice_lines.iter().enumerate()
            .map(|a| {
                convert_to_invoice_line_db(a.1, a.0 as i16)
            })
            .collect::<anyhow::Result<Vec<InvoiceLineDb>>>()?,
        additional_charges: req.additional_charges
            .iter()
            .enumerate()
            .map(|a| convert_to_additional_charge_db(a.1, a.0 as i16))
            .collect_vec(),
        financial_year: current_indian_financial_year() as i16,
        total_taxable_amount: req.total_taxable_amount()?,
        total_tax_amount: req.total_tax_amount()?,
        total_additional_charges_amount: req.total_additional_charge_amount(),
        round_off: 0.0,
        total_payable_amount: req.total_amount(currency_scale)?,
        created_by,
        igst_applicable,
        invoice_remarks: req.invoice_remarks.as_ref()
            .map(|a| a.get_str()),
        ecommerce_gstin: req.ecommerce_gstin
            .as_ref().map(|a| a.get_str()),
    })
}

fn compute_32_bit_xx_hash(st: &str) -> i64 {
    let mut hasher = xxh32::Xxh32::new(0);
    hasher.update(st.as_bytes());
    hasher.digest() as i64
}