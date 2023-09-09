use crate::hsc_sac::GstItemCode;
use gstin_validator::gstin_models::{validate_gstin, GstinValidationError};
use std::ops::Not;
use thiserror::Error;
use crate::invoice_line1::TaxPercentageError::NotInBounds;
use crate::invoice_line1::UnitPriceError::Negative;
use crate::invoice_line::line_number::LineNumber;
use crate::invoice_line::line_quantity::LineQuantity;
use crate::invoice_line::line_subtitle::LineSubtitle;
use crate::invoice_line::line_title::LineTitle;

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
pub struct UnitPrice(f64);
#[derive(Debug, Error)]
pub enum UnitPriceError {
    #[error("unit price cannot be less than  0.0")]
    Negative(f64),
    #[error("unit price cannot be larger than {0}")]
    TooLarge(f64),
}
impl UnitPrice {
    pub fn new(unit_price: f64) -> Result<Self, UnitPriceError> {
        if unit_price < 0.0 {
            return Err(Negative(unit_price));
        }
        if unit_price > 1_000_000_000.00 {
            return Err(UnitPriceError::TooLarge(1_000_000_000.00));
        }
        Ok(Self(unit_price))
    }
}
#[derive(Debug)]
pub struct TaxPercentage(f64);
#[derive(Debug, Error)]
pub enum TaxPercentageError {
    #[error("tax percentage {0} cannot be less than 0 or greater than 100")]
    NotInBounds(f64),
}
impl TaxPercentage {
    pub fn new(tax_percentage: f64) -> Result<Self, TaxPercentageError> {
        if !(0.0..=100.00).contains(&tax_percentage) {
            return Err(NotInBounds(tax_percentage));
        }
        Ok(Self(tax_percentage))
    }
}
#[derive(Debug)]
pub struct CessPercentage(f64);
#[derive(Debug, Error)]
pub enum CessPercentageError {
    #[error("cess percentage {0} cannot be less than 0 or greater than 500")]
    NotInBounds(f64),
}

impl CessPercentage {
    pub fn new(cess_percentage: f64) -> Result<Self, CessPercentageError> {
        if !(0.0..=500.00).contains(&cess_percentage) {
            return Err(CessPercentageError::NotInBounds(cess_percentage));
        }
        Ok(Self(cess_percentage))
    }
}
#[derive(Debug)]
pub struct DiscountPercentage(f64);
#[derive(Debug, Error)]
pub enum DiscountPercentageError {
    #[error("discount percentage {0} cannot be less than 0 or greater than 100")]
    NotInBounds(f64),
}
impl DiscountPercentage {
    pub fn new(discount_percentage: f64) -> Result<Self, DiscountPercentageError> {
        if !(0.0..=100.00).contains(&discount_percentage) {
            return Err(DiscountPercentageError::NotInBounds(discount_percentage));
        }
        Ok(DiscountPercentage(discount_percentage))
    }
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
