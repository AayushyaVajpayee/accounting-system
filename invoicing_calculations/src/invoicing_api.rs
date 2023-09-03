
use crate::invoicing_api::InvoiceLineError::{
    CessPercentageNegative, CessPercentageTooLarge, OuantityTooLarge, QuantityNegative,
    TaxPercentageNotInBounds, UnitPriceNegative, UnitPriceToolarge,
};
use std::ops::Not;
use thiserror::Error;

#[derive(Debug)]
pub struct InvoiceLine {
    quantity: f64,
    unit_price: f64,
    discount_percentage: f64,
    tax_percentage: f64,
    cess_percentage: f64,
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
impl InvoiceLine {
    pub fn new(
        quantity: f64,
        unit_price: f64,
        discount_percentage: f64,
        tax_percentage: f64,
        cess_percentage: f64,
    ) -> Result<Self, Vec<InvoiceLineError>> {
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
        if cess_percentage < 0.0 {
            vec.push(CessPercentageNegative);
        }
        if cess_percentage > 500.00 {
            vec.push(CessPercentageTooLarge(500.00))
        }
        if vec.is_empty().not() {
            Err(vec)
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

pub fn compute_discount_amount(line: &InvoiceLine) -> f64 {
    line.quantity * line.unit_price * line.discount_percentage / 100.00
}

pub fn compute_taxable_amount(line: &InvoiceLine) -> f64 {
    line.quantity * line.unit_price * (100.0 - line.discount_percentage) / 100.00
}

pub fn compute_tax_amount(line: &InvoiceLine) -> f64 {
    compute_taxable_amount(line) * line.tax_percentage / 100.0
}

pub fn compute_cess_amount(line: &InvoiceLine) -> f64 {
    compute_taxable_amount(line) * line.cess_percentage / 100.00
}

pub fn compute_line_total_amount(line: &InvoiceLine) -> f64 {
    compute_taxable_amount(line) + compute_tax_amount(line) + compute_cess_amount(line)
}

#[cfg(test)]
mod tests {
    use crate::invoicing_api::{compute_cess_amount, compute_discount_amount, compute_line_total_amount, compute_tax_amount, compute_taxable_amount, InvoiceLine};
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::FloatAssertions;

    #[rstest]
    #[case(InvoiceLine::new(0.0, 0.0, 0.0, 0.0, 0.0).unwrap(),0.0)]
    #[case(InvoiceLine::new(1.0, 1.0, 0.0, 0.0, 0.0).unwrap(),0.0)]
    #[case(InvoiceLine::new(1.0, 100.0, 10.0, 0.0, 0.0).unwrap(),10.0)]
    #[case(InvoiceLine::new(0.0, 0.0, 10.0, 0.0, 0.0).unwrap(),0.0)]
    #[case(InvoiceLine::new(100.0, 100.0, 10.0, 0.0, 0.0).unwrap(),1000.0)]
    #[case(InvoiceLine::new(1_000_000_000.0, 1_000_000_000.0, 10.0, 0.0, 0.0).unwrap()
    ,100_000_000_000_000_000.0)]
    fn test_compute_discount_amount(#[case] line: InvoiceLine, #[case] discount: f64) {
        let p = compute_discount_amount(&line);
        assert_that!(p).is_equal_to(discount);
    }
    #[rstest]
    #[case(InvoiceLine::new(0.0, 0.0, 0.0, 0.0, 0.0).unwrap(),0.0)]
    #[case(InvoiceLine::new(1.0, 1.0, 0.0, 0.0, 0.0).unwrap(),1.0)]
    #[case(InvoiceLine::new(1.0, 1.0, 1.0, 0.0, 0.0).unwrap(),0.99)]
    #[case(InvoiceLine::new(1.0, 0.0, 1.0, 0.0, 0.0).unwrap(),0.0)]
    #[case(InvoiceLine::new(1_000_000_000.00
    , 1_000_000_000.00, 1.0, 0.0, 0.0).unwrap(),990_000_000_000_000_000.00)]
    fn test_compute_taxable_amount(#[case] line:InvoiceLine,#[case] taxable_amount:f64){
        let p = compute_taxable_amount(&line);
        assert_that!(p).is_equal_to(taxable_amount);
    }
    #[rstest]
    #[case(InvoiceLine::new(0.0, 0.0, 0.0, 0.0, 0.0).unwrap(),0.0)]
    #[case(InvoiceLine::new(0.0, 0.0, 0.0, 10.0, 0.0).unwrap(),0.0)]
    #[case(InvoiceLine::new(1.0, 1.0, 0.0, 10.0, 0.0).unwrap(),0.1)]
    #[case(InvoiceLine::new(1_000_000_000.0, 1_000_000_000.0, 0.0, 10.0, 0.0).unwrap(),1_000_000_000_000_000_00.0)]
    fn test_compute_tax_amount(#[case] line:InvoiceLine,#[case] tax_amount:f64){
        let p = compute_tax_amount(&line);
        assert_that!(p).is_equal_to(tax_amount);
    }
    #[rstest]
    #[case(InvoiceLine::new(0.0, 0.0, 0.0, 0.0, 0.0).unwrap(),0.0)]
    #[case(InvoiceLine::new(0.0, 0.0, 0.0, 0.0, 10.0).unwrap(),0.0)]
    #[case(InvoiceLine::new(1.0, 1.0, 0.0, 0.0, 10.0).unwrap(),0.1)]
    #[case(InvoiceLine::new(1_000_000_000.0, 1_000_000_000.0, 0.0, 0.0, 10.0).unwrap(),1_000_000_000_000_000_00.0)]
    fn test_compute_cess_amount(#[case] line:InvoiceLine,#[case] cess_amount:f64){
        let p = compute_cess_amount(&line);
        assert_that!(p).is_equal_to(cess_amount);
    }
    #[rstest]
    #[case(InvoiceLine::new(0.0, 0.0, 0.0, 0.0, 0.0).unwrap(),0.0)]
    #[case(InvoiceLine::new(0.0, 0.0, 01.0, 01.0, 01.0).unwrap(),0.0)]
    #[case(InvoiceLine::new(100.0, 0.0, 01.0, 01.0, 01.0).unwrap(),0.0)]
    #[case(InvoiceLine::new(100.0, 1.0, 01.0, 01.0, 01.0).unwrap(),100.98)]
    #[case(InvoiceLine::new(1_000_000_000.0, 1_000_000_000.0, 0.0, 40.0, 300.0).unwrap()
    ,4_400_000_000_000_000_000.00)]
    fn test_line_total_amount(#[case] line:InvoiceLine,#[case] total_amount:f64){
        let p = compute_line_total_amount(&line);
        assert_that!(p).
            is_close_to(total_amount,0.0000000000001);
    }

}


