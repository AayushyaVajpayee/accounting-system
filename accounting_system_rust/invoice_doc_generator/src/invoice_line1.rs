use anyhow::bail;
use gstin_validator::gstin_models::{GstinValidationError, validate_gstin};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::hsc_sac::GstItemCode;
use crate::invoice_line::line_number::LineNumber;
use crate::invoice_line::line_quantity::LineQuantity;
use crate::invoice_line::line_subtitle::LineSubtitle;
use crate::invoice_line::line_title::LineTitle;
use crate::invoice_line::unit_price::Price;
use crate::percentages::tax_discount_cess::{CessPercentage, DiscountPercentage, TaxPercentage};

//length at most 100 char
//todo why do we need to own the string. we only need to read it

//length at most 50 char per line and max 2 lines

#[derive(Debug, Serialize, Deserialize,Clone)]
#[serde(try_from = "String")]
pub enum UOM {
    MilliLitre,
    Litre,
    Gram,
    KiloGram,
    Quintal,
    Piece,
    Box,
}

impl TryFrom<String> for UOM {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let k: UOM = match value.as_str().trim().to_lowercase().as_str() {
            "millilitre" => UOM::MilliLitre,
            "litre" => UOM::Litre,
            "gram" => UOM::Gram,
            "kilogram" => UOM::KiloGram,
            "quintal" => UOM::Quintal,
            "piece" => UOM::Piece,
            "box" => UOM::Box,
            _ => bail!("invalid uom value")
        };
        Ok(k)
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
#[allow(dead_code)]
#[derive(Debug)]
pub struct InvoiceLine {
    line_number: LineNumber,
    title: LineTitle,
    //length at most 100 char
    subtitle: LineSubtitle,
    //length at most 50 char per line and max 2 lines
    hsn_sac_code: GstItemCode,
    quantity: LineQuantity,
    discount_percentage: DiscountPercentage,
    unit_price: Price,
    tax_percentage: TaxPercentage,
    cess_percentage: CessPercentage,
}
