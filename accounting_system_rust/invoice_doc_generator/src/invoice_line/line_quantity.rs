use anyhow::bail;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::invoice_line::line_quantity::LineQuantityError::{NegativeValue, TooLarge};
use crate::invoice_line1::UOM;

#[derive(Debug, Serialize, Deserialize)]
pub struct LineQuantityRaw {
    quantity: f64,
    uom: String,
}

#[derive(Debug, Serialize, Deserialize, Builder, Clone)]
#[serde(try_from = "LineQuantityRaw")]
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
    pub fn get_quantity(&self) -> f64 {
        self.quantity
    }
    pub fn uom_as_str(&self) -> &str {
        self.uom.as_str()
    }
}

impl TryFrom<LineQuantityRaw> for LineQuantity {
    type Error = anyhow::Error;

    fn try_from(value: LineQuantityRaw) -> Result<Self, Self::Error> {
        let uom: UOM = value.uom.try_into()?;
        if value.quantity <= 0.0 {
            bail!("quantity cannot be less than 0")
        }
        if value.quantity >= 10_000_000.00 {
            bail!("quantity cannot be more than 1_00_00_000")
        }
        Ok(LineQuantity {
            quantity: value.quantity,
            uom,
        })
    }
}

#[cfg(test)]
mod line_quantity_tests {
    use rstest::rstest;
    use speculoos::assert_that;
    use speculoos::prelude::ResultAssertions;

    use crate::invoice_line::line_quantity::LineQuantity;
    use crate::invoice_line1::UOM;

    #[rstest]
    #[case(34.0, true)]
    #[case(- 34.0, false)]
    #[case(9999999999999999999999999.0, false)]
    fn test_line_quantity(#[case] input: f64, #[case] valid: bool) {
        let q = LineQuantity::new(input, UOM::Piece);
        if valid {
            assert_that!(q).is_ok();
        } else {
            assert_that!(q).is_err();
        }
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use crate::invoice_line::line_quantity::{LineQuantity, LineQuantityBuilder};
    use crate::invoice_line1::UOM;

    pub fn a_line_quantity(builder: LineQuantityBuilder) -> LineQuantity {
        LineQuantity {
            quantity: builder.quantity.unwrap_or(1.0),
            uom: builder.uom.unwrap_or(UOM::Piece),
        }
    }
}
