use crate::hsn_code_generated::HSN_SET;
use crate::invoice_line::LineNumberError::ShouldBeGreaterThan0;
use crate::invoice_line::LineQuantityError::{NegativeValue, TooLarge};
use crate::invoice_line::LineSubtitleError::Empty;
use crate::invoice_line::LineTitleError::{EmptyTitle, TooLong};
use crate::invoice_line::TaxPercentageError::NotInBounds;
use crate::invoice_line::UnitPriceError::Negative;
use gstin_validator::gstin_models::{validate_gstin, GstinValidationError};
use std::ops::Not;
use thiserror::Error;
use crate::hsc_sac::GstItemCode;

#[derive(Debug)]
pub struct LineNumber(u16);

#[derive(Debug, Error)]
pub enum LineNumberError {
    #[error("line number {0} should start from 1")]
    ShouldBeGreaterThan0(u16),
}
impl LineNumber {
    pub fn new(line_number: u16) -> Result<Self, LineNumberError> {
        if line_number == 0 {
            return Err(ShouldBeGreaterThan0(line_number));
        }
        Ok(Self(line_number))
    }
}

//length at most 100 char
//todo why do we need to own the string. we only need to read it
#[derive(Debug)]
pub struct LineTitle(String);
#[derive(Debug, Error)]
pub enum LineTitleError {
    #[error("line title cannot be empty")]
    EmptyTitle,
    #[error("line title should not be more than {0} char")]
    TooLong(u16),
}

impl LineTitle {
    pub fn new(title: String) -> Result<Self, LineTitleError> {
        if title.is_empty() {
            return Err(EmptyTitle);
        }
        if title.chars().count() >= 80 {
            return Err(TooLong(80));
        }
        Ok(Self(title))
    }
}

//length at most 50 char per line and max 2 lines
#[derive(Debug)]
pub struct LineSubtitle(String);
#[derive(Debug, Error)]
pub enum LineSubtitleError {
    #[error("line subtitle cannot be empty")]
    Empty,
    #[error("line subtitle should not be more than {0} char")]
    TooLong(u16),
}
impl LineSubtitle {
    pub fn new(subtitle: String) -> Result<Self, LineSubtitleError> {
        if subtitle.is_empty() {
            return Err(Empty);
        }
        if subtitle.chars().count() >= 100 {
            return Err(LineSubtitleError::TooLong(100));
        }
        Ok(Self(subtitle))
    }
}

#[derive(Debug)]
pub struct LineQuantity {
    quantity: f64,
    uom: UOM,
}
#[derive(Debug, Error)]
pub enum LineQuantityError {
    #[error("quantity {0} cannot be negative")]
    NegativeValue(f64),
    #[error("quantity cannot be larger than {0}")]
    TooLarge(f64),
}

impl LineQuantity {
    pub fn new(quantity: f64, uom: UOM) -> Result<Self, LineQuantityError> {
        if quantity < 0.0 {
            return Err(NegativeValue(quantity));
        }
        if quantity > 1_000_000_000.00 {
            return Err(TooLarge(1_000_000_000.00));
        }
        Ok(Self { quantity, uom })
    }
}
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
