use thiserror::Error;
use crate::percentages::tax_discount_cess::TaxPercentageError::NotInBounds;

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

#[cfg(test)]
mod tax_percentage_tests {
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;
    use crate::percentages::tax_discount_cess::TaxPercentage;

    #[rstest]
    #[case(-1.0,false)]
    #[case(-1010.0,false)]
    #[case(101.0, false)]
    #[case(0.0, true)]
    #[case(20.0, true)]
    fn test_tax_percentage(#[case] input: f64, #[case] valid: bool) {
        let p = TaxPercentage::new(input);
        if valid{
            assert_that!(p).is_ok();
        }else{
            assert_that!(p).is_err();
        }
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

#[cfg(test)]
mod cess_percentage_tests {
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;
    use crate::percentages::tax_discount_cess::CessPercentage;

    #[rstest]
    #[case(-1.0,false)]
    #[case(-1010.0,false)]
    #[case(5010.0,false)]
    #[case(101.0, true)]
    #[case(0.0, true)]
    #[case(20.0, true)]
    fn test_cess_percentage(#[case] input: f64, #[case] valid: bool) {
        let p = CessPercentage::new(input);
        if valid{
            assert_that!(p).is_ok();
        }else{
            assert_that!(p).is_err();
        }
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

#[cfg(test)]
mod discount_percentage_tests {
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;
    use crate::percentages::tax_discount_cess::DiscountPercentage;

    #[rstest]
    #[case(-1.0,false)]
    #[case(-1010.0,false)]
    #[case(101.0, false)]
    #[case(0.0, true)]
    #[case(20.0, true)]
    fn test_discount_percentage(#[case] input: f64, #[case] valid: bool) {
        let p = DiscountPercentage::new(input);
        if valid{
            assert_that!(p).is_ok();
        }else{
            assert_that!(p).is_err();
        }
    }
}