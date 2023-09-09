use crate::invoice_line1::{GstinNo, InvoiceLine, PaymentTerms, TaxPercentage, UnitPrice};
use crate::invoice_number::InvoiceNumber;

mod invoice_model;

mod hsn_code_generated;
mod sac_code_generated;
mod invoice_number;
mod hsc_sac;
mod invoice_line1;
mod invoice_line;


pub struct InvoiceHeader{
    invoice_number:InvoiceNumber,
    order_number:String,
    order_date:String,
    payment_terms:PaymentTerms,
    supplier_detail:SupplierDetail,
    customer_detail:CustomerDetail
}

pub struct SupplierDetail{
    gstin:Option<GstinNo>,
    address:Address,
}

pub struct CustomerDetail{
    gstin:Option<GstinNo>,
    billing_address:Address,
    shipping_address:Address
}

pub struct AdditionalCharge {
    unit_price: UnitPrice,
    tax_percent: TaxPercentage,
}


pub struct Address{
    city:String,
    pincode:String,
    address_line_1:String,
    address_line_2:String,
    address_line_3:String
}

pub struct Invoice{
    currency_name:String,
    header:InvoiceHeader,
    invoice_lines:Vec<InvoiceLine>,
    additional_charges:Vec<AdditionalCharge>,

}