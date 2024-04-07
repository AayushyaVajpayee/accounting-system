use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use cess_models::CessStrategy;
use crate::send_request;
use crate::util::generate_random_string;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTaxRateRequest {
    pub tax_rate_percentage: f32,
    pub start_date: DateTime<Utc>, 
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCessRequest {
    pub cess_strategy: CessStrategy,
    pub start_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateProductRequest {
    idempotence_key: Uuid,
    line_title: String,
    line_subtitle: String,
    hsn_sac_code: String,
    uom: String,
    //Piece
    create_tax_request: CreateTaxRateRequest,
    create_cess_request: Option<CreateCessRequest>,
}

async fn create_product(request: &CreateProductRequest, tenant_id: Uuid, user_id: Uuid) -> Uuid {
    send_request(request, tenant_id, user_id, "product-item/create").await
}

pub async fn create_random_product(tenant_id: Uuid, user_id: Uuid)->Uuid{
    let product_request = CreateProductRequest {
        idempotence_key: Uuid::now_v7(),
        line_title: generate_random_string(40),
        line_subtitle:  generate_random_string(40),
        hsn_sac_code: "01013020".to_string(),
        uom: "Piece".to_string(),
        create_tax_request: CreateTaxRateRequest {
            tax_rate_percentage: 12.0,
            start_date: Utc::now(),
        },
        create_cess_request: None,
    };
    create_product(&product_request,tenant_id,user_id).await
}