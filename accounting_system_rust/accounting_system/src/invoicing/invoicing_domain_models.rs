use uuid::Uuid;
use invoice_doc_generator::invoice_line::line_number::LineNumber;
use invoice_doc_generator::invoice_line::line_quantity::LineQuantity;
use invoice_doc_generator::invoice_line::unit_price::Price;
use invoice_doc_generator::invoice_number::InvoiceNumber;
use invoice_doc_generator::percentages::tax_discount_cess::{CessPercentage, DiscountPercentage, GSTPercentage};
use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::invoicing::invoicing_request_models::{BatchNo, ExpiryDateMs, PurchaseOrderNo};
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;

#[derive(Debug)]
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
#[derive(Debug)]
struct AdditionalCharge{
    id:Uuid,
    tenant_id:Uuid,
    invoice_table_id:Uuid,
    line_no:i16,
    line_title_id:Uuid,
    rate:i32,
    audit_metadata:AuditMetadataBase,
}
#[derive(Debug)]
struct Invoice{
    base_master_fields:BaseMasterFields,
    invoicing_series_mst_id:Uuid,
    financial_year:i16,
    invoice_number:InvoiceNumber,
    currency_id:Uuid,
    service_invoice:bool,
    invoice_date_ms:i64,
    e_invoicing_applicable:bool,
    supplier_business_entity_id:Uuid,
    b2b_invoice:bool,
    billed_to_business_entity_id:Option<Uuid>,
    shipped_to_business_entity_id:Option<Uuid>,
    purchase_order_number:Option<PurchaseOrderNo>,
    einvoice_json_s3_id:Option<Uuid>,
    total_taxable_amount:f64,
    total_tax_amount:f64,
    round_off:f64,
    total_payable_amount:f64,
    invoice_pdf_s3_id:Option<Uuid>,
    invoice_template_id:Option<Uuid>,
    payment_term_id:Option<Uuid>,
    audit_metadata:AuditMetadataBase
}

#[derive(Debug)]
struct InvoiceTemplate{
    base_master_fields:BaseMasterFields,
    sample_doc_s3_id:Option<Uuid>,
    //there will be a template id of the pdf generator to use in kotlin pdf generating service
    audit_metadata:AuditMetadataBase
}