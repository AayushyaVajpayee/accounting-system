use thiserror::Error;
use crate::invoice_line::unit_price::UnitPriceError::Negative;

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

#[cfg(test)]
mod unit_price_tests{
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;
    use crate::invoice_line::unit_price::UnitPrice;

    #[rstest]
    #[case(-1.0,false)]
    #[case(100_000_000_000.0,false)]
    #[case(-0.002,false)]
    #[case(0.023,true)]
    fn test_unit_price(#[case] input:f64,#[case] valid:bool){
        let p = UnitPrice::new(input);
        if valid{
            assert_that!(p).is_ok();
        }else{
            assert_that!(p).is_err();
        }
    }
}