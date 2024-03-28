use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use invoice_doc_generator::hsc_sac::GstItemCode;
use invoice_doc_generator::invoice_line::line_subtitle::LineSubtitle;
use invoice_doc_generator::invoice_line::line_title::LineTitle;
use invoice_doc_generator::percentages::tax_discount_cess::TaxPercentage;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::common_utils::pg_util::pg_util::create_composite_type_db_row;
use crate::common_utils::pg_util::pg_util::ToPostgresString;
use crate::common_utils::utils::get_current_indian_standard_time;
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;
use crate::masters::company_master::company_master_models::master_status_enum::MasterStatusEnum;
use crate::masters::company_master::company_master_models::master_updation_remarks::MasterUpdationRemarks;
use crate::masters::product_item_master::product_item_models::{CessStrategy, ProductCreationRequest};

#[derive(Debug)]
pub struct ProductItemDb<'a> {
    pub idempotence_key: Uuid,
    pub tenant_id: Uuid,
    pub created_by: Uuid,
    pub title: &'a str,
    pub subtitle: Option<&'a str>,
    pub product_hash: String,
    pub uom: &'a str,
    pub hsn_sac_code: &'a str,
    pub create_tax_rate_request: CreateTaxRateRequestDb,
    pub create_cess_request: CreateCessRequestDb<'a>,
}

#[derive(Debug)]
pub struct CreateTaxRateRequestDb {
    idempotence_key: Uuid,
    tenant_id: Uuid,
    created_by: Uuid,
    tax_percentage: f32,
    start_date: DateTime<Tz>,
}

#[derive(Debug)]
pub struct CreateCessRequestDb<'a> {
    idempotence_key: Uuid,
    tenant_id: Uuid,
    created_by: Uuid,
    cess_strategy: &'a str,
    cess_rate_percentage: f32,
    cess_amount_per_unit: f64,
    retail_sale_price: f64,
    //keep it zero when not required
    start_date: DateTime<Tz>,
}

impl ToPostgresString for CreateCessRequestDb<'_> {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        let fields: &[&dyn ToPostgresString] = &[
            &self.idempotence_key,
            &self.tenant_id,
            &self.created_by,
            &self.cess_strategy,
            &self.cess_rate_percentage,
            &self.cess_amount_per_unit,
            &self.retail_sale_price,
            //keep it zero when not required
            &self.start_date,
        ];
        create_composite_type_db_row(fields, f)
    }

    fn db_type_name(&self) -> &'static str {
        "create_product_item_request"
    }
}

impl ToPostgresString for CreateTaxRateRequestDb {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        let fields: &[&dyn ToPostgresString] = &[
            &self.idempotence_key,
            &self.tenant_id,
            &self.created_by,
            &self.tax_percentage,
            &self.start_date,
        ];
        create_composite_type_db_row(fields, f)
    }

    fn db_type_name(&self) -> &'static str {
        "create_tax_rate_request"
    }
}

impl ToPostgresString for ProductItemDb<'_> {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        let fields: &[&dyn ToPostgresString] = &[
            &self.idempotence_key,
            &self.tenant_id,
            &self.created_by,
            &self.title,
            &self.subtitle,
            &self.product_hash.as_str(),
            &self.uom,
            &self.hsn_sac_code,
            &self.create_tax_rate_request,
            &self.create_cess_request,
        ];
        create_composite_type_db_row(fields, f)
    }

    fn db_type_name(&self) -> &'static str {
        "create_product_item_request"
    }
}

fn convert_product_creation_request_to_cess_rate_db(req: &ProductCreationRequest,
                                                    tenant_id: Uuid,
                                                    created_by: Uuid)
                                                    -> CreateCessRequestDb {
    req.create_cess_request.as_ref()
        .map(|a| CreateCessRequestDb {
            idempotence_key: req.idempotence_key,
            tenant_id,
            created_by,
            cess_strategy: a.cess_strategy.get_strategy_name(),
            cess_rate_percentage: a.cess_strategy.get_cess_rate_percentage()
                .unwrap_or(0.0),
            cess_amount_per_unit: a.cess_strategy.get_cess_amount_per_unit()
                .unwrap_or(0.0),
            retail_sale_price: a.cess_strategy.get_retail_sale_price()
                .unwrap_or(0.0),
            start_date: a.start_date,
        })
        .unwrap_or_else(|| {
            CreateCessRequestDb {
                idempotence_key: req.idempotence_key,
                tenant_id,
                created_by,
                cess_strategy: CessStrategy::get_default_strategy_name(),
                cess_rate_percentage: 0.0,
                cess_amount_per_unit: 0.0,
                retail_sale_price: 0.0,
                start_date: get_current_indian_standard_time(),
            }
        })
}

fn convert_product_creation_request_to_create_tax_request(req: &ProductCreationRequest,
                                                          tenant_id: Uuid,
                                                          created_by: Uuid) -> CreateTaxRateRequestDb {
    req.create_tax_rate_request.as_ref()
        .map(|a| CreateTaxRateRequestDb {
            idempotence_key: req.idempotence_key,
            tenant_id,
            created_by,
            tax_percentage: a.tax_rate_percentage.inner(),
            start_date: a.start_date,
        }).unwrap_or_else(|| {
        CreateTaxRateRequestDb {
            idempotence_key: req.idempotence_key,
            tenant_id,
            created_by,
            tax_percentage: 0.0,
            start_date: get_current_indian_standard_time(),
        }
    })
}

pub fn convert_product_creation_request_to_product_item_db(req: &ProductCreationRequest,
                                                           tenant_id: Uuid,
                                                           created_by: Uuid) -> ProductItemDb {
    let mut hasher = Sha256::new();
    hasher.update(req.line_title.inner().as_bytes());
    if let Some(a) = req.line_subtitle.as_ref() {
        hasher.update(a.inner().as_bytes());
    }
    let result = hasher.finalize();
    let product_hash = format!("{:x}", result);

    ProductItemDb {
        idempotence_key: req.idempotence_key,
        tenant_id,
        created_by,
        title: req.line_title.inner(),
        subtitle: req.line_subtitle.as_ref().map(|a| a.inner()),
        product_hash,
        uom: req.uom.as_str(),
        hsn_sac_code: req.hsn_sac_code.as_str(),
        create_tax_rate_request: convert_product_creation_request_to_create_tax_request(req,
                                                                                        tenant_id,
                                                                                        created_by),
        create_cess_request: convert_product_creation_request_to_cess_rate_db(req,
                                                                              tenant_id,
                                                                              created_by),
    }
}
#[derive(Debug)]
pub struct ProductTaxRatDbResponse {
    pub tax_rate_percentage: TaxPercentage,
    pub start_date: DateTime<Tz>,
    pub end_date: Option<DateTime<Tz>>,
}
#[derive(Debug)]
pub struct CessTaxRateDbResponse {
    pub cess_strategy: CessStrategy,
    pub start_date: DateTime<Tz>,
    pub end_date: Option<DateTime<Tz>>,
}
#[derive(Debug)]

pub struct ProductItemDbResponse {
    pub base_master_fields: BaseMasterFields,
    pub title: LineTitle,
    pub subtitle: Option<LineSubtitle>,
    pub hsn_sac_code: GstItemCode,
    pub product_hash: String,
    pub temporal_tax_rates: Vec<ProductTaxRatDbResponse>,
    pub temporal_cess_rates: Vec<CessTaxRateDbResponse>,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductItemDbRawRsp {
    id: Uuid,
    tenant_id: Uuid,
    entity_version_id: i32,
    active: bool,
    approval_status: i16,
    remarks: Option<String>,
    title: String,
    subtitle: Option<String>,
    hsn_sac_code: String,
    created_by: Uuid,
    updated_by: Uuid,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaxRateDbRawRsp {
    tax_rate_percentage: f32,
    start_date: DateTime<Utc>,
    end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CessRateDbRawRsp {
    cess_strategy: String,
    cess_rate_percentage: f32,
    cess_amount_per_unit: f64,
    retail_sale_price: f64,
    start_date: DateTime<Utc>,
    end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetProductItemDbRsp {
    product_item: ProductItemDbRawRsp,
    temporal_tax_rates: Vec<TaxRateDbRawRsp>,
    temporal_cess_rates: Vec<CessRateDbRawRsp>,
}

pub fn convert_db_resp_to_product_item_db_resp(value: Value) -> anyhow::Result<ProductItemDbResponse> {
    let pi: GetProductItemDbRsp = serde_json::from_value(value)?;
    Ok(
        ProductItemDbResponse {
            base_master_fields: BaseMasterFields {
                id: pi.product_item.id,
                entity_version_id: pi.product_item.entity_version_id,
                tenant_id: pi.product_item.tenant_id,
                active: pi.product_item.active,
                approval_status: MasterStatusEnum::
                get_enum_for_value(pi
                    .product_item
                    .approval_status as usize
                )?,
                remarks: pi.product_item.remarks.as_ref()
                    .map(|a| MasterUpdationRemarks::new(a)).transpose()?,
            },
            title: LineTitle::new(pi.product_item.title)?,
            subtitle: pi.product_item.subtitle
                .map(|a| LineSubtitle::new(a))
                .transpose()?,
            hsn_sac_code: GstItemCode::new(pi.product_item.hsn_sac_code)?,
            product_hash: "".to_string(),
            temporal_tax_rates: pi.temporal_tax_rates
                .into_iter()
                .map(|tax_rt_db| {
                    let tax_perc = TaxPercentage::new(tax_rt_db.tax_rate_percentage);
                    tax_perc.map(|tp| ProductTaxRatDbResponse {
                        tax_rate_percentage: tp,
                        start_date: tax_rt_db.start_date.with_timezone(&Tz::Asia__Kolkata),
                        end_date: tax_rt_db.end_date.map(|d| d.with_timezone(&Tz::Asia__Kolkata)),
                    })
                })
                .try_collect()?,
            temporal_cess_rates: pi.temporal_cess_rates
                .into_iter()
                .map(|cess_db| {
                    CessStrategy::new(
                        cess_db.cess_strategy.as_str(),
                        cess_db.cess_rate_percentage,
                        cess_db.retail_sale_price,
                        cess_db.cess_amount_per_unit,
                    ).map(|st| CessTaxRateDbResponse {
                        cess_strategy: st,
                        start_date: cess_db.start_date.with_timezone(&Tz::Asia__Kolkata),
                        end_date: cess_db.end_date.map(|d| d.with_timezone(&Tz::Asia__Kolkata)),
                    })
                })
                .try_collect()?,
            audit_metadata: AuditMetadataBase {
                created_by: pi.product_item.created_by,
                updated_by: pi.product_item.updated_by,
                created_at: pi.product_item.created_at,
                updated_at: pi.product_item.updated_at,
            },
        }
    )
}
