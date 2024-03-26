use chrono::DateTime;
use chrono_tz::Tz;
use sha2::{Digest, Sha256};
use tracing_subscriber::fmt::format;
use uuid::Uuid;

use crate::common_utils::pg_util::pg_util::create_composite_type_db_row;
use crate::common_utils::pg_util::pg_util::ToPostgresString;
use crate::common_utils::utils::get_current_indian_standard_time;
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
    if let Some(a) = req.line_subtitle.as_ref(){
        hasher.update(a.inner().as_bytes());
    }
    let result = hasher.finalize();
    let product_hash = format!("{:x}",result);

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