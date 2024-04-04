use anyhow::bail;
use gstin_validator::gstin_models::{GstinValidationError, validate_gstin};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::hsn_sac::GstItemCode;
use crate::invoice_line::line_number::LineNumber;
use crate::invoice_line::line_quantity::LineQuantity;
use crate::invoice_line::line_subtitle::LineSubtitle;
use crate::invoice_line::line_title::LineTitle;
use crate::invoice_line::unit_price::Price;
use crate::percentages::tax_discount_cess::{CessPercentage, DiscountPercentage, TaxPercentage};

//length at most 100 char
//todo why do we need to own the string. we only need to read it

//length at most 50 char per line and max 2 lines

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl UOM {
    pub fn as_str(&self) -> &str {
        match self {
            UOM::MilliLitre => { "MilliLitre" }
            UOM::Litre => { "Litre" }
            UOM::Gram => { "Gram" }
            UOM::KiloGram => { "KiloGram" }
            UOM::Quintal => { "Quintal" }
            UOM::Piece => { "Piece" }
            UOM::Box => { "Box" }
        }
    }
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

//length at most 100 char
//todo why do we need to own the string. we only need to read it

//length at most 50 char per line and max 2 lines

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "String")]
pub enum EInvoicingUOM {
    BAG,
    BAL,
    BDL,
    BKL,
    BOU,
    BOX,
    BTL,
    BUN,
    CAN,
    CCM,
    CMS,
    CBM,
    CTN,
    DOZ,
    DRM,
    GGK,
    GMS,
    GRS,
    GYD,
    KGS,
    KLR,
    KME,
    LTR,
    MLS,
    MLT,
    MTR,
    MTS,
    NOS,
    OTH,
    PAC,
    PCS,
    PRS,
    QTL,
    ROL,
    SET,
    SQF,
    SQM,
    SQY,
    TBS,
    TGM,
    THD,
    TON,
    TUB,
    UGS,
    UNT,
    YDS,
}

impl EInvoicingUOM {
    pub fn as_str(&self) -> &str {
        match self {
            EInvoicingUOM::BAG => { "BAGS" }
            EInvoicingUOM::BAL => { "BALE" }
            EInvoicingUOM::BDL => { "BUNDLES" }
            EInvoicingUOM::BKL => { "BUCKLES" }
            EInvoicingUOM::BOU => { "BILLION OF UNITS" }
            EInvoicingUOM::BOX => { "BOX" }
            EInvoicingUOM::BTL => { "BOTTLES" }
            EInvoicingUOM::BUN => { "BUNCHES" }
            EInvoicingUOM::CAN => { "CANS" }
            EInvoicingUOM::CCM => { "CUBIC CENTIMETERS" }
            EInvoicingUOM::CMS => { "CENTIMETERS" }
            EInvoicingUOM::CBM => { "CUBIC METERS" }
            EInvoicingUOM::CTN => { "CARTONS" }
            EInvoicingUOM::DOZ => { "DOZENS" }
            EInvoicingUOM::DRM => { "DRUMS" }
            EInvoicingUOM::GGK => { "GREAT GROSS" }
            EInvoicingUOM::GMS => { "GRAMMES" }
            EInvoicingUOM::GRS => { "GROSS" }
            EInvoicingUOM::GYD => { "GROSS YARDS" }
            EInvoicingUOM::KGS => { "KILOGRAMS" }
            EInvoicingUOM::KLR => { "KILOLITRE" }
            EInvoicingUOM::KME => { "KILOMETRE" }
            EInvoicingUOM::LTR => { "LITRES" }
            EInvoicingUOM::MLS => { "MILLI LITRES" }
            EInvoicingUOM::MLT => { "MILILITRE" }
            EInvoicingUOM::MTR => { "METERS" }
            EInvoicingUOM::MTS => { "METRIC TON" }
            EInvoicingUOM::NOS => { "NUMBERS" }
            EInvoicingUOM::OTH => { "OTHERS" }
            EInvoicingUOM::PAC => { "PACKS" }
            EInvoicingUOM::PCS => { "PIECES" }
            EInvoicingUOM::PRS => { "PAIRS" }
            EInvoicingUOM::QTL => { "QUINTAL" }
            EInvoicingUOM::ROL => { "ROLLS" }
            EInvoicingUOM::SET => { "SETS" }
            EInvoicingUOM::SQF => { "SQUARE FEET" }
            EInvoicingUOM::SQM => { "SQUARE METERS" }
            EInvoicingUOM::SQY => { "SQUARE YARDS" }
            EInvoicingUOM::TBS => { "TABLETS" }
            EInvoicingUOM::TGM => { "TEN GROSS" }
            EInvoicingUOM::THD => { "THOUSANDS" }
            EInvoicingUOM::TON => { "TONNES" }
            EInvoicingUOM::TUB => { "TUBES" }
            EInvoicingUOM::UGS => { "US GALLONS" }
            EInvoicingUOM::UNT => { "UNITS" }
            EInvoicingUOM::YDS => { "YARDS" }
        }
    }
}

impl From<&UOM> for EInvoicingUOM {
    fn from(value: &UOM) -> Self {
        match value {
            UOM::MilliLitre => { EInvoicingUOM::MLT }
            UOM::Litre => { EInvoicingUOM::LTR }
            UOM::Gram => { EInvoicingUOM::GMS }
            UOM::KiloGram => { EInvoicingUOM::KGS }
            UOM::Quintal => { EInvoicingUOM::QTL }
            UOM::Piece => { EInvoicingUOM::PCS }
            UOM::Box => { EInvoicingUOM::BOX }
        }
    }
}

impl TryFrom<String> for EInvoicingUOM {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let k: EInvoicingUOM = match value.as_str().trim().to_uppercase().as_str() {
            "BAGS" => EInvoicingUOM::BAG,
            "BALE" => EInvoicingUOM::BAL,
            "BUNDLES" => EInvoicingUOM::BDL,
            "BUCKLES" => EInvoicingUOM::BKL,
            "BILLION OF UNITS" => EInvoicingUOM::BOU,
            "BOX" => EInvoicingUOM::BOX,
            "BOTTLES" => EInvoicingUOM::BTL,
            "BUNCHES" => EInvoicingUOM::BUN,
            "CANS" => EInvoicingUOM::CAN,
            "CUBIC CENTIMETERS" => EInvoicingUOM::CCM,
            "CENTIMETERS" => EInvoicingUOM::CMS,
            "CUBIC METERS" => EInvoicingUOM::CBM,
            "CARTONS" => EInvoicingUOM::CTN,
            "DOZENS" => EInvoicingUOM::DOZ,
            "DRUMS" => EInvoicingUOM::DRM,
            "GREAT GROSS" => EInvoicingUOM::GGK,
            "GRAMMES" => EInvoicingUOM::GMS,
            "GROSS" => EInvoicingUOM::GRS,
            "GROSS YARDS" => EInvoicingUOM::GYD,
            "KILOGRAMS" => EInvoicingUOM::KGS,
            "KILOLITRE" => EInvoicingUOM::KLR,
            "KILOMETRE" => EInvoicingUOM::KME,
            "LITRES" => EInvoicingUOM::LTR,
            "MILLI LITRES" => EInvoicingUOM::MLS,
            "MILILITRE" => EInvoicingUOM::MLT,
            "METERS" => EInvoicingUOM::MTR,
            "METRIC TON" => EInvoicingUOM::MTS,
            "NUMBERS" => EInvoicingUOM::NOS,
            "OTHERS" => EInvoicingUOM::OTH,
            "PACKS" => EInvoicingUOM::PAC,
            "PIECES" => EInvoicingUOM::PCS,
            "PAIRS" => EInvoicingUOM::PRS,
            "QUINTAL" => EInvoicingUOM::QTL,
            "ROLLS" => EInvoicingUOM::ROL,
            "SETS" => EInvoicingUOM::SET,
            "SQUARE FEET" => EInvoicingUOM::SQF,
            "SQUARE METERS" => EInvoicingUOM::SQM,
            "SQUARE YARDS" => EInvoicingUOM::SQY,
            "TABLETS" => EInvoicingUOM::TBS,
            "TEN GROSS" => EInvoicingUOM::TGM,
            "THOUSANDS" => EInvoicingUOM::THD,
            "TONNES" => EInvoicingUOM::TON,
            "TUBES" => EInvoicingUOM::TUB,
            "US GALLONS" => EInvoicingUOM::UGS,
            "UNITS" => EInvoicingUOM::UNT,
            "YARDS" => EInvoicingUOM::YDS,
            _ => bail!("invalid EInvoicingUOM value")
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
