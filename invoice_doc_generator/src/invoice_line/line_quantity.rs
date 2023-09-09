use crate::invoice_line::line_quantity::LineQuantityError::{NegativeValue, TooLarge};
use crate::invoice_line1::UOM;
use thiserror::Error;

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
#[cfg(test)]
mod line_quantity_tests {
    use rstest::rstest;
    use crate::invoice_line1::UOM;
    use crate::invoice_line::line_quantity::LineQuantity;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;

    #[rstest]
    #[case(34.0,true)]
    #[case(-34.0,false)]
    #[case(9999999999999999999999999.0,false)]
    fn test_line_quantity(#[case] input: f64, #[case] valid: bool) {
        let q = LineQuantity::new(input,UOM::Piece);
        if valid {
            assert_that!(q).is_ok();
        }else{
            assert_that!(q).is_err();
        }
    }
}
