use std::fmt::{Display, Formatter};
use std::ops::Not;

use anyhow::anyhow;
use thiserror::Error;

use cess_models::CessStrategy;

use crate::invoice_line::InvoiceLineError::{
    OuantityTooLarge, QuantityNegative,
    TaxPercentageNotInBounds, UnitPriceNegative, UnitPriceToolarge,
};

#[derive(Debug)]
pub struct InvoiceLine {
    quantity: f64,
    unit_price: f64,
    discount_percentage: f32,
    tax_percentage: f32,
    cess_percentage: CessStrategy,
}



#[derive(Debug, Error)]
pub enum InvoiceLineError {
    #[error("quantity cannot be negative")]
    QuantityNegative,
    #[error("quantity larger than {0} not supported")]
    OuantityTooLarge(f64),
    #[error("unit price cannot be negative")]
    UnitPriceNegative,
    #[error("unit price larger than {0} not supported")]
    UnitPriceToolarge(f64),
    #[error("tax percentage cannot be less than 0 or greater than 100")]
    TaxPercentageNotInBounds,
    #[error("cess percentage cannot be negative")]
    CessPercentageNegative,
    #[error("cess percentage larger than {0} is not valid")]
    CessPercentageTooLarge(f64),
    #[error("discount percentage cannot be less than 0 or greater than100")]
    DiscountPercentageNotInBounds,
}
#[derive(Debug)]
struct ErrorList( Vec<InvoiceLineError>);
impl Display for ErrorList{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (index, error) in self.0.iter().enumerate() {
            writeln!(f, "{}: {}", index + 1, error)?;
        }
        Ok(())
    }
}

impl InvoiceLine {
    pub fn new(
        quantity: f64,
        unit_price: f64,
        discount_percentage: f32,
        tax_percentage: f32,
        cess_percentage: CessStrategy,
    ) -> anyhow::Result<Self> {
        let mut vec: Vec<InvoiceLineError> = Vec::new();
        if quantity < 0.0 {
            vec.push(QuantityNegative);
        }
        if quantity > 1_000_000_000.00 {
            vec.push(OuantityTooLarge(1_000_000_000.00))
        }
        if unit_price < 0.0 {
            vec.push(UnitPriceNegative);
        }
        if unit_price > 1_000_000_000.00 {
            vec.push(UnitPriceToolarge(1_000_000_000.00))
        }
        if !(0.0..=100.0).contains(&tax_percentage) {
            vec.push(TaxPercentageNotInBounds);
        }

        if vec.is_empty().not() {
            Err(anyhow!(ErrorList(vec).to_string()))
        } else {
            Ok(Self {
                quantity,
                unit_price,
                discount_percentage,
                tax_percentage,
                cess_percentage,
            })
        }
    }
}
impl InvoiceLine{
    pub fn compute_discount_amount(&self) -> f64 {
        self.quantity * self.unit_price * (self.discount_percentage as f64) / 100.00
    }


    pub fn compute_taxable_amount(&self) -> f64 {
        self.quantity * self.unit_price * (100.0 - self.discount_percentage as f64) / 100.00
    }

    pub fn compute_tax_amount(&self) -> f64 {
        self.compute_taxable_amount() * (self.tax_percentage as f64) / 100.0
    }

    pub fn compute_cess_amount(&self) -> f64 {
       self.cess_percentage.calculate_cess_amount(self.compute_taxable_amount(),self.quantity)
    }

    pub fn compute_line_total_amount(&self) -> f64 {
       self.compute_taxable_amount() + self.compute_tax_amount() +self.compute_cess_amount()
    }

}


#[cfg(test)]
mod tests {
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::FloatAssertions;
    use cess_models::CessStrategy;
    use crate::invoice_line::InvoiceLine;

    #[rstest]
    #[case(InvoiceLine::new(0.0, 0.0, 0.0, 0.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 0.0)]
    #[case(InvoiceLine::new(1.0, 1.0, 0.0, 0.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 0.0)]
    #[case(InvoiceLine::new(1.0, 100.0, 10.0, 0.0,CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 10.0)]
    #[case(InvoiceLine::new(0.0, 0.0, 10.0, 0.0,CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 0.0)]
    #[case(InvoiceLine::new(100.0, 100.0, 10.0, 0.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 1000.0)]
    #[case(InvoiceLine::new(1_000_000_000.0, 1_000_000_000.0, 10.0, 0.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 100_000_000_000_000_000.0)]
    fn test_compute_discount_amount(#[case] line: InvoiceLine, #[case] discount: f64) {
        let p = line.compute_discount_amount();
        assert_that!(p).is_equal_to(discount);
    }

    #[rstest]
    #[case(InvoiceLine::new(0.0, 0.0, 0.0, 0.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 0.0)]
    #[case(InvoiceLine::new(1.0, 1.0, 0.0, 0.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 1.0)]
    #[case(InvoiceLine::new(1.0, 1.0, 1.0, 0.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 0.99)]
    #[case(InvoiceLine::new(1.0, 0.0, 1.0, 0.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 0.0)]
    #[case(InvoiceLine::new(1_000_000_000.00, 1_000_000_000.00, 1.0, 0.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 990_000_000_000_000_000.00)]
    fn test_compute_taxable_amount(#[case] line: InvoiceLine, #[case] taxable_amount: f64) {
        let p = line.compute_taxable_amount();
        assert_that!(p).is_equal_to(taxable_amount);
    }

    #[rstest]
    #[case(InvoiceLine::new(0.0, 0.0, 0.0, 0.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 0.0)]
    #[case(InvoiceLine::new(0.0, 0.0, 0.0, 10.0,CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 0.0)]
    #[case(InvoiceLine::new(1.0, 1.0, 0.0, 10.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 0.1)]
    #[case(InvoiceLine::new(1_000_000_000.0, 1_000_000_000.0, 0.0, 10.0,CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 1_000_000_000_000_000_00.0)]
    fn test_compute_tax_amount(#[case] line: InvoiceLine, #[case] tax_amount: f64) {
        let p = line.compute_tax_amount();
        assert_that!(p).is_equal_to(tax_amount);
    }

    #[rstest]
    #[case(InvoiceLine::new(0.0, 0.0, 0.0, 0.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 0.0)]
    #[case(InvoiceLine::new(0.0, 0.0, 0.0, 0.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 10.0}).unwrap(), 0.0)]
    #[case(InvoiceLine::new(1.0, 1.0, 0.0, 0.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 10.0}).unwrap(), 0.1)]
    #[case(InvoiceLine::new(1_000_000_000.0, 1_000_000_000.0, 0.0, 0.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 10.0}).unwrap(), 1_000_000_000_000_000_00.0)]
    fn test_compute_cess_amount(#[case] line: InvoiceLine, #[case] cess_amount: f64) {
        let p = line.compute_cess_amount();
        assert_that!(p).is_equal_to(cess_amount);
    }

    #[rstest]
    #[case(InvoiceLine::new(0.0, 0.0, 0.0, 0.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 0.0}).unwrap(), 0.0)]
    #[case(InvoiceLine::new(0.0, 0.0, 01.0, 01.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 01.0}).unwrap(), 0.0)]
    #[case(InvoiceLine::new(100.0, 0.0, 01.0, 01.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 01.0}).unwrap(), 0.0)]
    #[case(InvoiceLine::new(100.0, 1.0, 01.0, 01.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 01.0}).unwrap(), 100.98)]
    #[case(InvoiceLine::new(1_000_000_000.0, 1_000_000_000.0, 0.0, 40.0, CessStrategy::PercentageOfAssessableValue {cess_rate_percentage: 300.0}).unwrap()
    , 4_400_000_000_000_000_000.00)]
    fn test_line_total_amount(#[case] line: InvoiceLine, #[case] total_amount: f64) {
        let p = line.compute_line_total_amount();
        assert_that!(p).
            is_close_to(total_amount, 0.0000000000001);
    }
}


