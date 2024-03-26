use chrono::DateTime;
use chrono_tz::Tz;
use derive_builder::Builder;
use uuid::Uuid;

use invoice_doc_generator::hsc_sac::GstItemCode;
use invoice_doc_generator::invoice_line1::UOM;
use invoice_doc_generator::invoice_line::line_subtitle::LineSubtitle;
use invoice_doc_generator::invoice_line::line_title::LineTitle;
use invoice_doc_generator::invoice_line::unit_price::Price;
use invoice_doc_generator::percentages::tax_discount_cess::TaxPercentage;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;

#[derive(Debug, Builder)]
pub(crate) struct ProductItem {
    pub base_master_fields: BaseMasterFields,
    pub title: LineTitle,
    pub subtitle: Option<LineSubtitle>,
    pub uom: UOM,
    pub hsn_sac_code: GstItemCode,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug, Builder)]
pub(crate) struct ProductTaxRate {
    pub base_master_fields: BaseMasterFields,
    pub product_item_id: Uuid,
    pub tax_rate_percentage: TaxPercentage,
    pub start_date: DateTime<Tz>,
    pub end_date: Option<DateTime<Tz>>,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug, Builder)]
pub(crate) struct CessTaxRate {
    pub base_master_fields: BaseMasterFields,
    pub product_item_id: Uuid,
    pub cess_strategy: String,
    pub cess_rate_percentage: f32,
    pub cess_amount_per_unit: f64,
    pub retail_sale_price: Price,
    pub start_date: DateTime<Tz>,
    pub end_date: Option<DateTime<Tz>>,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug, Builder)]
pub struct ProductCreationRequest {
    pub idempotence_key: Uuid,
    pub line_title: LineTitle,
    pub line_subtitle: Option<LineSubtitle>,
    pub hsn_sac_code: GstItemCode,
    pub uom: UOM,
    pub create_tax_rate_request: Option<CreateTaxRateRequest>,
    pub create_cess_request: Option<CreateCessRequest>,
}

#[derive(Debug, Builder, Clone)]
pub struct CreateTaxRateRequest {
    pub tax_rate_percentage: TaxPercentage,
    pub start_date: DateTime<Tz>,//todo ensure that it is not in past more than 24 hours
}

#[derive(Debug, Builder, Clone)]
pub struct CreateCessRequest {
    pub cess_strategy: CessStrategy,
    pub start_date: DateTime<Tz>,//todo ensure that it is not in past more than 24 hours
}

///create tagged serialisation and deserialization so that there is no ambiguity
#[derive(Debug, Clone)]
pub enum CessStrategy {
    PercentageOfAssessableValue {
        cess_rate_percentage: f32
    },
    AmountPerUnit {
        cess_amount_per_unit: f64,
    },
    PercentageOfAssessableValueAndAmountPerUnit {
        cess_rate_percentage: f32,
        cess_amount_per_unit: f64,
    },
    MaxOfPercentageOfAssessableValueAndAmountPerUnit {
        cess_rate_percentage: f32,
        cess_amount_per_unit: f64,
    },
    PercentageOfRetailSalePrice {
        cess_rate_percentage: f32,
        retail_sale_price: f64,
    },
}
impl CessStrategy{
   pub fn get_strategy_name(&self)->&'static str{
        match self{
            CessStrategy::PercentageOfAssessableValue { .. } => {
                "percentage_of_assessable_value"
            }
            CessStrategy::AmountPerUnit { .. } => {
                "amount_per_unit"
            }
            CessStrategy::PercentageOfAssessableValueAndAmountPerUnit { .. } => {
                "percentage_of_assessable_value_and_amount_per_unit"
            }
            CessStrategy::MaxOfPercentageOfAssessableValueAndAmountPerUnit { .. } => {
                "max_of_percentage_of_assessable_value_and_amount_per_unit"
            }
            CessStrategy::PercentageOfRetailSalePrice { .. } => {
                "percentage_of_retail_sale_price"
            }
        }
    }
    pub fn get_default_strategy_name()->&'static str {
        "percentage_of_assessable_value"
    }
    //cess_rate_percentage: a,
    //             cess_amount_per_unit: 0.0,
    //             retail_sale_price: 0.0,
    pub fn get_cess_rate_percentage(&self) -> Option<f32> {
        match self {
            CessStrategy::PercentageOfAssessableValue { cess_rate_percentage, .. } => {
                Some(*cess_rate_percentage)
            }
            CessStrategy::AmountPerUnit { .. } => {
                None
            }
            CessStrategy::PercentageOfAssessableValueAndAmountPerUnit { cess_rate_percentage, .. } => {
                Some(*cess_rate_percentage)
            }
            CessStrategy::MaxOfPercentageOfAssessableValueAndAmountPerUnit { cess_rate_percentage, .. } => {
                Some(*cess_rate_percentage)
            }
            CessStrategy::PercentageOfRetailSalePrice { cess_rate_percentage, .. } => {
                Some(*cess_rate_percentage)
            }
        }
    }

    pub fn get_cess_amount_per_unit(&self) -> Option<f64> {
        match self {
            CessStrategy::PercentageOfAssessableValue { .. } => None,
            CessStrategy::AmountPerUnit { cess_amount_per_unit, .. } => {
                Some(*cess_amount_per_unit)
            }
            CessStrategy::PercentageOfAssessableValueAndAmountPerUnit { cess_amount_per_unit, .. } => {
                Some(*cess_amount_per_unit)
            }
            CessStrategy::MaxOfPercentageOfAssessableValueAndAmountPerUnit { cess_amount_per_unit, .. } => {
                Some(*cess_amount_per_unit)
            }
            CessStrategy::PercentageOfRetailSalePrice { .. } => None,
        }
    }

    pub fn get_retail_sale_price(&self) -> Option<f64> {
        match self {
            CessStrategy::PercentageOfAssessableValue { .. } => None,
            CessStrategy::AmountPerUnit { .. } => None,
            CessStrategy::PercentageOfAssessableValueAndAmountPerUnit { .. } => None,
            CessStrategy::MaxOfPercentageOfAssessableValueAndAmountPerUnit { .. } => None,
            CessStrategy::PercentageOfRetailSalePrice { retail_sale_price, .. } => {
                Some(*retail_sale_price)
            }
        }
    }
}