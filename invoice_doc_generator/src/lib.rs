use thiserror::Error;
use crate::LineNumberError::ShouldBeGreaterThan0;
use crate::LineSubtitleError::Empty;
use crate::LineTitleError::{EmptyTitle, TooLong};

mod invoice_model;

mod hsn_code_generated;
mod sac_code_generated;

pub struct LineNumber(u16);

#[derive(Debug,Error)]
pub enum LineNumberError{
    #[error("line number {0} should start from 1")]
    ShouldBeGreaterThan0(u16)
}
impl LineNumber{
    pub fn new(line_number:u16)->Result<Self,LineNumberError>{
        if line_number==0{
           return Err(ShouldBeGreaterThan0(line_number));
        }
        Ok(Self(line_number))
    }
}

//length at most 100 char
//todo why do we need to own the string. we only need to read it
pub struct LineTitle(String);
#[derive(Debug,Error)]
pub enum LineTitleError{
    #[error("line title cannot be empty")]
    EmptyTitle,
    #[error("line title should not be more than {0} char")]
    TooLong(u16),
}
impl LineTitle{
    pub fn new(title:String)->Result<Self,LineTitleError>{
        if title.is_empty(){
            return Err(EmptyTitle);
        }
        if title.chars().count()>=80{
            return Err(TooLong(80));
        }
        Ok(Self(title))
    }
}
//length at most 50 char per line and max 2 lines
pub struct LineSubtitle(String);
#[derive(Debug,Error)]
pub enum LineSubtitleError{
    #[error("line subtitle cannot be empty")]
    Empty,
    #[error("line subtitle should not be more than {0} char")]
    TooLong(u16)
}
impl LineSubtitle{
    pub fn new(subtitle:String)->Result<Self,LineSubtitleError>{
        if subtitle.is_empty(){
            return Err(Empty);
        }
        if subtitle.chars().count()>=100{
            return Err(LineSubtitleError::TooLong(100));
        }
        Ok(Self(subtitle))
    }
}


pub enum GstItemCode{
    HSN(Hsn),
    SAC(Sac)
}
pub struct Hsn(String);

#[derive(Debug,Error)]
pub enum HsnError{

}
impl Hsn{
    pub fn new(hsn:String)->Result<Self,HsnError>{

        Ok(Self(hsn))
    }
}
pub struct Sac(String);

pub struct LineQuantity{
    quantity:f64,
    uom:UOM
}
pub enum UOM{
    MilliLitre,
    Litre,
    Gram,
    KiloGram,
    Quintal,
    Piece,
    Box,
}

pub struct UnitPrice(f64);

pub struct TaxPercentage(f64);
pub struct CessPercentage(f64);
pub struct DiscountPercentage(f64);
pub struct GstinNo(String);
pub enum PaymentTerms{

}
pub struct InvoiceNumber(String);
pub struct InvoiceLine{
    line_number:LineNumber,
    title:LineTitle,//length at most 100 char
    subtitle:LineSubtitle,//length at most 50 char per line and max 2 lines
    hsn_sac_code:GstItemCode, //todo make it an enum of hsn and sac and validate it
    quantity:LineQuantity,//todo round everything to 2 decimal places max
    discount_percentage:DiscountPercentage,//todo will have to apply validations like not negative and rounding to 2 deimal places
    unit_price:UnitPrice,
    tax_percentage:TaxPercentage,
    cess_percentage:CessPercentage,
}

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