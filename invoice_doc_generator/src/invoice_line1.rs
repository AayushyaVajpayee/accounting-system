use crate::hsc_sac::GstItemCode;
use crate::invoice_line::line_number::LineNumber;
use crate::invoice_line::line_quantity::LineQuantity;
use crate::invoice_line::line_subtitle::LineSubtitle;
use crate::invoice_line::line_title::LineTitle;
use crate::invoice_line::unit_price::UnitPrice;
use gstin_validator::gstin_models::{validate_gstin, GstinValidationError};
use thiserror::Error;
use crate::percentages::tax_discount_cess::{CessPercentage, DiscountPercentage, TaxPercentage};

//length at most 100 char
//todo why do we need to own the string. we only need to read it

//length at most 50 char per line and max 2 lines

#[derive(Debug)]
pub enum UOM {
    MilliLitre,
    Litre,
    Gram,
    KiloGram,
    Quintal,
    Piece,
    Box,
}

#[derive(Debug)]
pub struct GstinNo(String);
#[derive(Debug, Error)]
pub enum GstinNoError {}
impl GstinNo {
    pub fn new(gstin: String) -> Result<Self, GstinValidationError> {
        let validation_errors = validate_gstin(gstin.as_str());
        if validation_errors.is_some() {
            return Err(validation_errors.unwrap());
        }
        Ok(GstinNo(gstin))
    }
}
#[derive(Debug)]
pub enum PaymentTerms {}

#[derive(Debug)]
pub struct InvoiceLine {
    line_number: LineNumber,
    title: LineTitle,                        //length at most 100 char
    subtitle: LineSubtitle,                  //length at most 50 char per line and max 2 lines
    hsn_sac_code: GstItemCode,               //todo make it an enum of hsn and sac and validate it
    quantity: LineQuantity,                  //todo round everything to 2 decimal places max
    discount_percentage: DiscountPercentage, //todo will have to apply validations like not negative and rounding to 2 deimal places
    unit_price: UnitPrice,
    tax_percentage: TaxPercentage,
    cess_percentage: CessPercentage,
}
