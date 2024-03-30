use crate::masters::product_item_master::product_item_models::CessStrategy;

impl CessStrategy {
    pub fn calculate_cess_amount(&self, taxable_value: f64, quantity: f64) -> f64 {
        match &self {
            CessStrategy::PercentageOfAssessableValue { cess_rate_percentage } => {
                ((*cess_rate_percentage as f64) / 100.00) * taxable_value
            }
            CessStrategy::AmountPerUnit { cess_amount_per_unit } => {
                cess_amount_per_unit * quantity
            }
            CessStrategy::PercentageOfAssessableValueAndAmountPerUnit { cess_rate_percentage, cess_amount_per_unit } => {
                (((*cess_rate_percentage as f64) / 100.00) * taxable_value) + cess_amount_per_unit * quantity
            }
            CessStrategy::MaxOfPercentageOfAssessableValueAndAmountPerUnit { cess_rate_percentage, cess_amount_per_unit } => {
                let p = ((*cess_rate_percentage as f64) / 100.00) * taxable_value;
                let d = cess_amount_per_unit * quantity;
                f64::max(p,d)
            }
            CessStrategy::PercentageOfRetailSalePrice { cess_rate_percentage, retail_sale_price } => {
                ((*cess_rate_percentage as f64) / 100.00) * retail_sale_price
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use rstest::rstest;
    use crate::masters::product_item_master::product_item_models::CessStrategy;

    #[rstest]
    #[case(CessStrategy::PercentageOfAssessableValue { cess_rate_percentage: 10.0 }, 1000.0, 2.0, 100.0)]
    #[case(CessStrategy::PercentageOfAssessableValue { cess_rate_percentage: 5.0 }, 2000.0, 1.0, 100.0)]
    fn test_percentage_of_assessable_value(#[case] strategy: CessStrategy, #[case] taxable_value: f64, #[case] quantity: f64, #[case] expected_cess_amount: f64) {
        let actual_cess_amount = strategy.calculate_cess_amount(taxable_value, quantity);
        assert_eq!(actual_cess_amount, expected_cess_amount);
    }

    #[rstest]
    #[case(CessStrategy::AmountPerUnit { cess_amount_per_unit: 5.0 }, 1000.0, 3.0, 15.0)]
    #[case(CessStrategy::AmountPerUnit { cess_amount_per_unit: 10.0 }, 500.0, 2.0, 20.0)]
    fn test_amount_per_unit(#[case] strategy: CessStrategy, #[case] taxable_value: f64, #[case] quantity: f64, #[case] expected_cess_amount: f64) {
        let actual_cess_amount = strategy.calculate_cess_amount(taxable_value, quantity);
        assert_eq!(actual_cess_amount, expected_cess_amount);
    }

    #[rstest]
    #[case(CessStrategy::PercentageOfAssessableValueAndAmountPerUnit { cess_rate_percentage: 5.0, cess_amount_per_unit: 10.0 }, 2000.0, 4.0, 140.0)]
    #[case(CessStrategy::PercentageOfAssessableValueAndAmountPerUnit { cess_rate_percentage: 3.0, cess_amount_per_unit: 8.0 }, 3000.0, 2.0, 106.0)]
    fn test_percentage_of_assessable_value_and_amount_per_unit(#[case] strategy: CessStrategy, #[case] taxable_value: f64, #[case] quantity: f64, #[case] expected_cess_amount: f64) {
        let actual_cess_amount = strategy.calculate_cess_amount(taxable_value, quantity);
        assert_eq!(actual_cess_amount, expected_cess_amount);
    }

    #[rstest]
    #[case(CessStrategy::MaxOfPercentageOfAssessableValueAndAmountPerUnit { cess_rate_percentage: 8.0, cess_amount_per_unit: 15.0 }, 1500.0, 2.0, 120.0)]
    #[case(CessStrategy::MaxOfPercentageOfAssessableValueAndAmountPerUnit { cess_rate_percentage: 6.0, cess_amount_per_unit: 20.0 }, 1500.0, 3.0, 90.0)]
    fn test_max_of_percentage_of_assessable_value_and_amount_per_unit(#[case] strategy: CessStrategy, #[case] taxable_value: f64, #[case] quantity: f64, #[case] expected_cess_amount: f64) {
        let actual_cess_amount = strategy.calculate_cess_amount(taxable_value, quantity);
        assert_eq!(actual_cess_amount, expected_cess_amount);
    }

    #[rstest]
    #[case(CessStrategy::PercentageOfRetailSalePrice { cess_rate_percentage: 12.0, retail_sale_price: 5000.0 }, 4000.0, 1.0, 600.0)]
    #[case(CessStrategy::PercentageOfRetailSalePrice { cess_rate_percentage: 8.0, retail_sale_price: 6000.0 }, 5000.0, 2.0, 480.0)]
    fn test_percentage_of_retail_sale_price(#[case] strategy: CessStrategy, #[case] taxable_value: f64, #[case] quantity: f64, #[case] expected_cess_amount: f64) {
        let actual_cess_amount = strategy.calculate_cess_amount(taxable_value, quantity);
        assert_eq!(actual_cess_amount, expected_cess_amount);
    }
}