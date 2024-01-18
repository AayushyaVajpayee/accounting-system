use std::ops::Not;
use thiserror::Error;

use crate::additional_charge::AdditionalChargeError::{
    RateNegative, RateTooLarge, TaxPercentageOutOfBounds,
};

#[derive(Debug)]
pub struct AdditionalCharge {
    rate: f64,
    tax_percentage: f64,
}

#[derive(Debug, Error)]
pub enum AdditionalChargeError {
    #[error("rate {0} cannot be negative")]
    RateNegative(f64),
    #[error("rate larger than {0} not supported")]
    RateTooLarge(f64),
    #[error("tax percentage cannot be less than 0 and greater than 100")]
    TaxPercentageOutOfBounds,
}

impl AdditionalCharge {
    pub fn new(rate: f64, tax_percentage: f64) -> Result<Self, Vec<AdditionalChargeError>> {
        let mut errors: Vec<AdditionalChargeError> = Vec::new();
        if rate < 0.0 {
            errors.push(RateNegative(rate));
        }
        if rate > 1_000_000_000.00 {
            errors.push(RateTooLarge(1_000_000_000.00));
        }
        if !(0.0..=100.00).contains(&tax_percentage) {
            errors.push(TaxPercentageOutOfBounds);
        }
        if errors.is_empty().not() {
            Err(errors)
        } else {
            Ok(Self {
                rate,
                tax_percentage,
            })
        }
    }
}

pub fn compute_tax_amount(additional_charge: &AdditionalCharge) -> f64 {
    additional_charge.tax_percentage * additional_charge.rate / 100.00
}
pub fn compute_total_charge_amount(additional_charge: &AdditionalCharge) -> f64 {
    additional_charge.rate * (additional_charge.tax_percentage + 100.00) / 100.00
}

#[cfg(test)]
mod tests{
    use rstest::rstest;
    use spectral::assert_that;

    use crate::additional_charge::{AdditionalCharge, compute_tax_amount, compute_total_charge_amount};

    #[rstest]
    #[case(AdditionalCharge::new(0.0, 0.0).unwrap(),0.0)]
    #[case(AdditionalCharge::new(100.0, 10.0).unwrap(),10.0)]
    #[case(AdditionalCharge::new(0.0, 10.0).unwrap(),0.0)]
    fn test_compute_tax_amount(#[case] line:AdditionalCharge,#[case] taxable_amount:f64){
        let p = compute_tax_amount(&line);
        assert_that!(p).is_equal_to(taxable_amount);
    }

    #[rstest]
    #[case(AdditionalCharge::new(0.0, 0.0).unwrap(),0.0)]
    #[case(AdditionalCharge::new(100.0, 10.0).unwrap(),110.0)]
    #[case(AdditionalCharge::new(0.0, 10.0).unwrap(),0.0)]
    fn test_compute_total_charge_amount(#[case] line:AdditionalCharge,#[case] total_amount:f64){
        let p = compute_total_charge_amount(&line);
        assert_that!(p).is_equal_to(total_amount);
    }

}