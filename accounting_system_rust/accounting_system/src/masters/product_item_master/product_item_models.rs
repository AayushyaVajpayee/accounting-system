use anyhow::Context;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use cess_models::CessStrategy;
use invoice_doc_generator::hsc_sac::GstItemCode;
use invoice_doc_generator::invoice_line1::UOM;
use invoice_doc_generator::invoice_line::line_subtitle::LineSubtitle;
use invoice_doc_generator::invoice_line::line_title::LineTitle;
use invoice_doc_generator::invoice_line::unit_price::Price;
use invoice_doc_generator::percentages::tax_discount_cess::{GSTPercentage, TaxPercentage};

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::common_utils::utils::get_current_indian_standard_time;
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;

#[derive(Debug, Serialize, Deserialize, Builder, Clone, PartialEq)]
pub struct ProductItemResponse {
    pub base_master_fields: BaseMasterFields,
    pub title: LineTitle,
    pub subtitle: Option<LineSubtitle>,
    pub hsn_sac_code: GstItemCode,
    pub product_hash: String,
    pub temporal_tax_rates: Vec<ProductTaxRateResponse>,
    pub temporal_cess_rates: Vec<CessTaxRateResponse>,
    pub audit_metadata: AuditMetadataBase,
}

impl ProductItemResponse {
    pub fn get_tax_rate(&self) -> anyhow::Result<&ProductTaxRateResponse> {
        let curr_time = get_current_indian_standard_time();
        let applicable_tax_rate = self.temporal_tax_rates
            .iter()
            .filter(|a| a.start_date.le(&curr_time))
            .max_by(|a, b| a.start_date.cmp(&b.start_date))
            .context("temporal tax rates cannot be empty")?;
        Ok(applicable_tax_rate)
    }
    pub fn get_cess_rate(&self) -> anyhow::Result<&CessTaxRateResponse> {
        let curr_time = get_current_indian_standard_time();
        let applicable_cess_rate = self.temporal_cess_rates
            .iter()
            .filter(|a| a.start_date.le(&curr_time))
            .max_by(|a, b| a.start_date.cmp(&b.start_date))
            .context("temporal tax rates cannot be empty")?;
        Ok(applicable_cess_rate)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Builder)]
pub struct ProductTaxRateResponse {
    pub tax_rate_percentage: GSTPercentage,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Builder)]
pub struct CessTaxRateResponse {
    pub cess_strategy: CessStrategy,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
}

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
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
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
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug, Builder, Deserialize, Serialize)]
pub struct ProductCreationRequest {
    pub idempotence_key: Uuid,
    pub line_title: LineTitle,
    pub line_subtitle: Option<LineSubtitle>,
    pub hsn_sac_code: GstItemCode,
    pub uom: UOM,
    pub create_tax_rate_request: Option<CreateTaxRateRequest>,
    pub create_cess_request: Option<CreateCessRequest>,
}

#[derive(Debug, Builder, Clone, Serialize, Deserialize)]
pub struct CreateTaxRateRequest {
    pub tax_rate_percentage: GSTPercentage,
    pub start_date: DateTime<Utc>,//todo ensure that it is not in past more than 24 hours
}

#[derive(Debug, Builder, Clone, Serialize, Deserialize)]
pub struct CreateCessRequest {
    pub cess_strategy: CessStrategy,
    pub start_date: DateTime<Utc>,//todo ensure that it is not in past more than 24 hours
}


#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use chrono::Utc;
    use lazy_static::lazy_static;
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use uuid::Uuid;

    use cess_models::CessStrategy;
    use invoice_doc_generator::hsc_sac::{GstItemCode, Hsn};
    use invoice_doc_generator::invoice_line1::UOM;
    use invoice_doc_generator::invoice_line::line_title::LineTitle;

    use crate::accounting::currency::currency_models::tests::an_audit_metadata_base;
    use crate::common_utils::utils::get_current_indian_standard_time;
    use crate::masters::company_master::company_master_models::base_master_fields::tests::a_base_master_field;
    use crate::masters::product_item_master::product_item_models::{CessTaxRateResponse, CessTaxRateResponseBuilder, ProductCreationRequest, ProductCreationRequestBuilder, ProductItemResponse, ProductItemResponseBuilder, ProductTaxRateResponse, ProductTaxRateResponseBuilder};

    lazy_static! {
        pub static ref SEED_PRODUCT_ITEM_ID:Uuid = Uuid::
        from_str("018e7b88-65d8-7545-85c4-b41146987929").unwrap();
    }

    pub fn a_product_creation_request(builder: ProductCreationRequestBuilder) -> ProductCreationRequest {
        let rng = rand::thread_rng();
        let title = rng.sample_iter(Alphanumeric)
            .take(19)
            .map(|a| a as char)
            .collect::<String>();
        ProductCreationRequest {
            idempotence_key: builder.idempotence_key.unwrap_or_else(Uuid::now_v7),
            line_title: builder.line_title.unwrap_or_else(|| LineTitle::new(title).unwrap()),
            line_subtitle: builder.line_subtitle.flatten(),
            hsn_sac_code: builder.hsn_sac_code.unwrap_or_else(|| GstItemCode::HsnCode(Hsn::new("38220011".to_string()).unwrap())),
            uom: builder.uom.unwrap_or(UOM::MilliLitre),
            create_tax_rate_request: builder.create_tax_rate_request.flatten(),
            create_cess_request: builder.create_cess_request.flatten(),
        }
    }

    pub fn a_product_item_response(builder: ProductItemResponseBuilder) -> ProductItemResponse {
        ProductItemResponse {
            base_master_fields: builder.base_master_fields
                .unwrap_or_else(|| a_base_master_field(Default::default())),
            title: builder.title
                .unwrap_or_else(|| LineTitle::new("some title".to_string()).unwrap()),
            subtitle: builder.subtitle.flatten(),
            hsn_sac_code: builder.hsn_sac_code.
                unwrap_or_else(|| GstItemCode::HsnCode(Hsn::new("38220011".to_string()).unwrap())),
            product_hash: builder.product_hash.unwrap_or_else(|| "hash".to_string()),
            temporal_tax_rates: builder.temporal_tax_rates.unwrap_or(vec![a_product_tax_rate_response(Default::default())]),
            temporal_cess_rates: builder.temporal_cess_rates.unwrap_or(vec![a_cess_rate_response(Default::default())]),
            audit_metadata: builder.audit_metadata.unwrap_or_else(|| an_audit_metadata_base(Default::default())),
        }
    }

    fn a_product_tax_rate_response(builder: ProductTaxRateResponseBuilder) -> ProductTaxRateResponse {
        ProductTaxRateResponse {
            tax_rate_percentage: builder.tax_rate_percentage.unwrap_or(5.0.try_into().unwrap()),
            start_date: builder.start_date.unwrap_or(get_current_indian_standard_time()),
            end_date: builder.end_date.flatten(),
        }
    }

    fn a_cess_rate_response(builder: CessTaxRateResponseBuilder) -> CessTaxRateResponse {
        CessTaxRateResponse {
            cess_strategy: builder.cess_strategy.unwrap_or(CessStrategy::PercentageOfAssessableValue { cess_rate_percentage: 0.0 }),
            start_date: builder.start_date.unwrap_or(get_current_indian_standard_time()),
            end_date: None,
        }
    }

    #[test]
    fn test_get_tax_rate() {
        let test_cases = vec![
            (
                vec![
                    ProductTaxRateResponse {
                        tax_rate_percentage: 5.0.try_into().unwrap(),
                        start_date: Utc::now() - chrono::Duration::days(1),
                        end_date: None,
                    },
                    ProductTaxRateResponse {
                        tax_rate_percentage: 12.0.try_into().unwrap(),
                        start_date: Utc::now() + chrono::Duration::days(1),
                        end_date: None,
                    },
                ],
                5.0,
            ),
            (
                vec![
                    ProductTaxRateResponse {
                        tax_rate_percentage: 5.0.try_into().unwrap(),
                        start_date: Utc::now() - chrono::Duration::days(1),
                        end_date: None,
                    },
                    ProductTaxRateResponse {
                        tax_rate_percentage: 12.0.try_into().unwrap(),
                        start_date: Utc::now() + chrono::Duration::days(1),
                        end_date: None,
                    },
                ],
                5.0,
            ),
            (
                vec![
                    ProductTaxRateResponse {
                        tax_rate_percentage: 5.0.try_into().unwrap(),
                        start_date: Utc::now(),
                        end_date: None,
                    },
                    ProductTaxRateResponse {
                        tax_rate_percentage: 12.0.try_into().unwrap(),
                        start_date: Utc::now() + chrono::Duration::days(2),
                        end_date: None,
                    },
                ],
                5.0,
            ),
        ];

        for (temporal_tax_rates, expected_tax_rate) in test_cases {
            let mut builder = ProductItemResponseBuilder::default();
            builder.temporal_tax_rates(temporal_tax_rates);
            let product_item_response = a_product_item_response(builder);

            let applicable_tax_rate = product_item_response.get_tax_rate().unwrap();
            assert_eq!(applicable_tax_rate.tax_rate_percentage, expected_tax_rate.try_into().unwrap());
        }
    }

    #[test]
    fn test_get_cess_rate() {
        let test_cases = vec![
            (
                vec![
                    CessTaxRateResponse {
                        cess_strategy: CessStrategy::PercentageOfAssessableValue {
                            cess_rate_percentage: 2.0,
                        },
                        start_date: Utc::now() - chrono::Duration::days(1),
                        end_date: None,
                    },
                    CessTaxRateResponse {
                        cess_strategy: CessStrategy::PercentageOfAssessableValue {
                            cess_rate_percentage: 3.0,
                        },
                        start_date: Utc::now() + chrono::Duration::days(1),
                        end_date: None,
                    },
                ],
                2.0,
            ),
            (
                vec![
                    CessTaxRateResponse {
                        cess_strategy: CessStrategy::PercentageOfAssessableValue {
                            cess_rate_percentage: 2.0,
                        },
                        start_date: Utc::now(),
                        end_date: None,
                    },
                    CessTaxRateResponse {
                        cess_strategy: CessStrategy::PercentageOfAssessableValue {
                            cess_rate_percentage: 3.0,
                        },
                        start_date: Utc::now() + chrono::Duration::days(2),
                        end_date: None,
                    },
                ],
                2.0,
            ),
        ];

        for (temporal_cess_rates, expected_cess_rate) in test_cases {
            let mut builder = ProductItemResponseBuilder::default();
            builder.temporal_cess_rates(temporal_cess_rates);
            let product_item_response = a_product_item_response(builder);
            let applicable_cess_rate = product_item_response.get_cess_rate().unwrap();
            assert_eq!(applicable_cess_rate.cess_strategy.get_cess_rate_percentage().unwrap(), expected_cess_rate);
        }
    }
}