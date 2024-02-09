use derive_builder::Builder;
use uuid::Uuid;

use invoice_doc_generator::invoice_line::line_number::LineNumber;
use invoice_doc_generator::invoice_line::line_quantity::LineQuantity;
use invoice_doc_generator::invoice_line::unit_price::Price;
use invoice_doc_generator::invoice_number::InvoiceNumber;
use invoice_doc_generator::percentages::tax_discount_cess::{CessPercentage, DiscountPercentage, GSTPercentage};

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::invoicing::invoicing_request_models::{BatchNo, ExpiryDateMs, PurchaseOrderNo};
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;

#[derive(Debug, Builder)]
pub struct InvoiceLine {
    pub base_master_fields: BaseMasterFields,
    pub invoice_table_id: Uuid,
    pub line_title_hsn_sac_id: Uuid,
    pub line_subtitle_id: Option<Uuid>,
    pub quantity: LineQuantity,
    pub unit_price: Price,
    pub tax_rate_bps: GSTPercentage,
    pub discount_percentage: DiscountPercentage,
    pub cess_percentage: CessPercentage,
    pub line_no: LineNumber,
    pub line_total: f64,
    pub mrp: Option<Price>,
    pub batch_no: Option<BatchNo>,
    pub expiry_date_ms: Option<ExpiryDateMs>,
    pub igst_applicable: bool,
    pub audit_metadata: AuditMetadataBase,
}
#[allow(dead_code)]
#[derive(Debug,Builder)]
struct AdditionalCharge {
    id: Uuid,
    tenant_id: Uuid,
    invoice_table_id: Uuid,
    line_no: i16,
    line_title_id: Uuid,
    rate: i32,
    audit_metadata: AuditMetadataBase,
}

#[derive(Debug,Builder)]
struct Invoice {
    base_master_fields: BaseMasterFields,
    invoicing_series_mst_id: Uuid,
    financial_year: i16,
    invoice_number: InvoiceNumber,
    currency_id: Uuid,
    service_invoice: bool,
    invoice_date_ms: i64,
    e_invoicing_applicable: bool,
    supplier_business_entity_id: Uuid,
    b2b_invoice: bool,
    billed_to_business_entity_id: Option<Uuid>,
    shipped_to_business_entity_id: Option<Uuid>,
    purchase_order_number: Option<PurchaseOrderNo>,
    einvoice_json_s3_id: Option<Uuid>,
    total_taxable_amount: f64,
    total_tax_amount: f64,
    round_off: f64,
    total_payable_amount: f64,
    invoice_pdf_s3_id: Option<Uuid>,
    invoice_template_id: Option<Uuid>,
    payment_term_id: Option<Uuid>,
    audit_metadata: AuditMetadataBase,
}
#[allow(dead_code)]
#[derive(Debug)]
struct InvoiceTemplate {
    base_master_fields: BaseMasterFields,
    sample_doc_s3_id: Option<Uuid>,
    //there will be a template id of the pdf generator to use in kotlin pdf generating service
    audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
pub mod tests {
    use uuid::Uuid;

    use invoice_doc_generator::invoice_line::line_number::LineNumber;
    use invoice_doc_generator::invoice_line::line_quantity::test_utils::a_line_quantity;
    use invoice_doc_generator::invoice_line::unit_price::Price;
    use invoice_doc_generator::invoice_number::InvoiceNumber;
    use invoice_doc_generator::percentages::tax_discount_cess::{CessPercentage, DiscountPercentage, GSTPercentage};
    use crate::accounting::currency::currency_models::tests::SEED_CURRENCY_ID;
    use crate::accounting::currency::currency_models::tests::an_audit_metadata_base;
    use crate::common_utils::utils::{current_indian_financial_year, get_current_time_us};
    use crate::invoicing::invoicing_domain_models::{AdditionalCharge, AdditionalChargeBuilder, Invoice, InvoiceBuilder, InvoiceLine, InvoiceLineBuilder};
    use crate::invoicing::invoicing_request_models::tests::SEED_INVOICE_ID;
    use crate::invoicing::invoicing_series::invoicing_series_models::tests::SEED_INVOICING_SERIES_MST_ID;
    use crate::invoicing::line_subtitle::line_subtitle_models::tests::SEED_SUBTITLE_ID;
    use crate::invoicing::line_title::line_title_models::tests::SEED_LINE_TITLE_HSN_ID;
    use crate::masters::business_entity_master::business_entity_models::tests::{SEED_BUSINESS_ENTITY_ID2, SEED_BUSINESS_ENTITY_INVOICE_DTL_ID1};
    use crate::masters::company_master::company_master_models::base_master_fields::tests::a_base_master_field;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    pub fn an_invoice_line(builder: InvoiceLineBuilder) -> InvoiceLine {
        InvoiceLine {
            base_master_fields: builder.base_master_fields.unwrap_or_else(|| a_base_master_field(Default::default())),
            invoice_table_id: builder.invoice_table_id.unwrap_or(*SEED_INVOICE_ID),
            line_title_hsn_sac_id: builder.line_title_hsn_sac_id.unwrap_or(*SEED_LINE_TITLE_HSN_ID),
            line_subtitle_id: builder.line_subtitle_id.unwrap_or(Some(*SEED_SUBTITLE_ID)),
            quantity: builder.quantity.unwrap_or(a_line_quantity(Default::default())),
            unit_price: builder.unit_price.unwrap_or(Price::new(10.0).unwrap()),
            tax_rate_bps: builder.tax_rate_bps.unwrap_or(GSTPercentage::new(28).unwrap()),
            discount_percentage: builder.discount_percentage.unwrap_or(DiscountPercentage::new(0.0).unwrap()),
            cess_percentage: builder.cess_percentage.unwrap_or(CessPercentage::new(0.0).unwrap()),
            line_no: builder.line_no.unwrap_or(LineNumber::new(1).unwrap()),
            line_total: 0.0,
            mrp: None,
            batch_no: None,
            expiry_date_ms: None,
            igst_applicable: false,
            audit_metadata: an_audit_metadata_base(Default::default()),
        }
    }

    pub fn an_additional_charge(builder:AdditionalChargeBuilder)->AdditionalCharge{
        AdditionalCharge{
            id: builder.id.unwrap_or_else(Uuid::now_v7),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            invoice_table_id: builder.invoice_table_id.unwrap_or(*SEED_INVOICE_ID),
            line_no: builder.line_no.unwrap_or(1),
            line_title_id: builder.line_title_id.unwrap_or(*SEED_LINE_TITLE_HSN_ID),
            rate: builder.rate.unwrap_or(0),
            audit_metadata: an_audit_metadata_base(Default::default()),
        }
    }
    #[allow(dead_code)]
    pub fn an_invoice(builder:InvoiceBuilder)->Invoice{
        Invoice{
            base_master_fields: builder.base_master_fields.unwrap_or_else(||a_base_master_field(Default::default())),
            invoicing_series_mst_id: builder.invoicing_series_mst_id.unwrap_or(*SEED_INVOICING_SERIES_MST_ID),
            financial_year: builder.financial_year.unwrap_or_else(||current_indian_financial_year() as i16),
            invoice_number: builder.invoice_number.unwrap_or_else(||InvoiceNumber::new("ABC123".to_string()).unwrap()),
            currency_id: builder.currency_id.unwrap_or(*SEED_CURRENCY_ID),
            service_invoice: builder.service_invoice.unwrap_or(false),
            invoice_date_ms: builder.invoice_date_ms.unwrap_or_else(||get_current_time_us().unwrap()/1000),
            e_invoicing_applicable: builder.e_invoicing_applicable.unwrap_or(false),
            supplier_business_entity_id: builder.supplier_business_entity_id.unwrap_or(*SEED_BUSINESS_ENTITY_INVOICE_DTL_ID1),
            b2b_invoice: builder.b2b_invoice.unwrap_or(false),
            billed_to_business_entity_id: builder.billed_to_business_entity_id.unwrap_or(Some(*SEED_BUSINESS_ENTITY_ID2)),
            shipped_to_business_entity_id: builder.shipped_to_business_entity_id.unwrap_or( Some(*SEED_BUSINESS_ENTITY_ID2)),
            purchase_order_number: builder.purchase_order_number.flatten(),
            einvoice_json_s3_id: builder.einvoice_json_s3_id.flatten(),
            total_taxable_amount:builder.total_taxable_amount.unwrap_or( 5.0),
            total_tax_amount: builder.total_tax_amount.unwrap_or(1.0),
            round_off: builder.round_off.unwrap_or(0.0),
            total_payable_amount: builder.total_payable_amount.unwrap_or(6.0),
            invoice_pdf_s3_id: builder.invoice_pdf_s3_id.flatten(),
            invoice_template_id: builder.invoice_template_id.flatten(),
            payment_term_id: builder.payment_term_id.flatten(),
            audit_metadata: builder.audit_metadata.unwrap_or(an_audit_metadata_base(Default::default())),
        }
    }
}