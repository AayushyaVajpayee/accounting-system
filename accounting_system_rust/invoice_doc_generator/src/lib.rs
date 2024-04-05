use crate::invoice_line::unit_price::Price;
use crate::invoice_line1::{GstinNo, InvoiceLine, PaymentTerms};
use crate::invoice_number::InvoiceNumber;
use crate::percentages::tax_discount_cess::TaxPercentage;

pub mod hsn_code_generated;
pub mod hsn_sac;
pub mod invoice_line;
pub mod invoice_line1;
mod invoice_model;
pub mod invoice_number;
pub mod percentages;
mod sac_code_generated;

#[allow(dead_code)]
pub struct InvoiceHeader {
    invoice_number: InvoiceNumber,
    order_number: String,
    order_date: String,
    payment_terms: PaymentTerms,
    supplier_detail: SupplierDetail,
    customer_detail: CustomerDetail,
}

#[allow(dead_code)]
pub struct SupplierDetail {
    gstin: Option<GstinNo>,
    address: Address,
}

#[allow(dead_code)]
pub struct CustomerDetail {
    gstin: Option<GstinNo>,
    billing_address: Address,
    shipping_address: Address,
}

#[allow(dead_code)]
pub struct AdditionalCharge {
    unit_price: Price,
    tax_percent: TaxPercentage, // lets not tax additional charge just show it.
}

#[allow(dead_code)]
pub struct Address {
    city: String,
    pincode: String,
    address_line_1: String,
    address_line_2: String,
    address_line_3: String,
}

#[allow(dead_code)]
pub struct Invoice {
    currency_name: String,
    header: InvoiceHeader,
    invoice_lines: Vec<InvoiceLine>,
    additional_charges: Vec<AdditionalCharge>,
}
