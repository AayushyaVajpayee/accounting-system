use std::marker::PhantomData;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::invoice_line1::UOM;
use crate::invoice_line::line_quantity::LineQuantityError::{NegativeValue, TooLarge};

pub type FreeLineQuantity = BaseLineQuantity<FreeLineQuantityTag>;
pub type LineQuantity = BaseLineQuantity<LineQuantityTag>;
#[derive(Debug,Clone)]
pub struct FreeLineQuantityTag;
#[derive(Debug,Clone)]
pub struct LineQuantityTag;

#[derive(Debug, Serialize, Deserialize, Builder, Clone)]
pub struct BaseLineQuantity<T> {
    quantity: f64,
    uom: UOM,
    _phantom: PhantomData<T>,
}

impl<T> BaseLineQuantity<T> {
    fn create_type(quantity: f64, uom: UOM) -> Result<Self, LineQuantityError> {
        if quantity > 1_000_000_000.00 {
            return Err(TooLarge(1_000_000_000.00));
        }
        Ok(Self {
            quantity,
            uom,
            _phantom: PhantomData,
        })
    }

    pub fn get_quantity(&self) -> f64 {
        self.quantity
    }

    pub fn uom_as_str(&self) -> &str {
        self.uom.as_str()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineQuantityRaw {
    quantity: f64,
    uom: String,
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
        if quantity <= 0.0 {
            return Err(NegativeValue(quantity));
        }
        BaseLineQuantity::create_type(quantity, uom)
    }
}

impl FreeLineQuantity {
    pub fn new(quantity: f64, uom: UOM) -> Result<Self, LineQuantityError> {
        if quantity < 0.0 {
            return Err(NegativeValue(quantity));
        }
        BaseLineQuantity::create_type(quantity, uom)
    }
}

#[cfg(test)]
mod line_quantity_tests {
    use rstest::rstest;
    use speculoos::assert_that;
    use speculoos::prelude::ResultAssertions;

    use crate::invoice_line1::UOM;
    use crate::invoice_line::line_quantity::LineQuantity;

    #[rstest]
    #[case(34.0, true)]
    #[case(- 34.0, false)]
    #[case(9999999999999999999999999.0, false)]
    fn test_line_quantity(#[case] input: f64, #[case] valid: bool) {
        let q = LineQuantity::new(input, UOM::Piece);
        if valid {
            assert_that!(q).is_ok();
            assert!(q.is_ok())
        } else {
            assert!(q.is_err())
        }
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use std::marker::PhantomData;

    use crate::invoice_line1::UOM;
    use crate::invoice_line::line_quantity::{BaseLineQuantityBuilder, FreeLineQuantity, FreeLineQuantityTag, LineQuantity, LineQuantityTag};

    pub fn a_line_quantity(builder: BaseLineQuantityBuilder<LineQuantityTag>) -> LineQuantity {
        LineQuantity {
            quantity: builder.quantity.unwrap_or(1.0),
            uom: builder.uom.unwrap_or(UOM::Piece),
            _phantom: PhantomData::default(),
        }
    }

    pub fn a_free_line_quantity(builder: BaseLineQuantityBuilder<FreeLineQuantityTag>) -> FreeLineQuantity {
        FreeLineQuantity {
            quantity: builder.quantity.unwrap_or(1.0),
            uom: builder.uom.unwrap_or(UOM::Piece),
            _phantom: Default::default(),
        }
    }
}
