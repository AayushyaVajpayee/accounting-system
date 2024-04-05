use chrono::{DateTime, Utc};
use std::str::FromStr;

use cess_models::CessStrategy;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const LOCAL_HOST: &str = "http://localhost:8090/";

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTenantRequest {
    pub idempotence_key: Uuid,
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub idempotence_key: Uuid,
    pub tenant_id: Uuid,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email_id: Option<String>,
    pub mobile_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCurrencyMasterRequest {
    pub idempotence_key: Uuid,
    pub scale: i16,
    pub display_name: String,
    pub description: String,
}

lazy_static! {
    pub static ref SUPER_TENANT_ID: Uuid =
        Uuid::from_str("018b33d9-c862-7fde-a0cd-55504d75e5e9").unwrap();
}

lazy_static! {
    pub static ref SUPER_USER_ID: Uuid =
        Uuid::from_str("018b3444-dc75-7a3f-a4d9-02c41071d3bd").unwrap();
}

async fn get_create_tenant_request(request: &CreateTenantRequest) -> Uuid {
    let mut cli = reqwest::Client::new();
    let path = format!("{}tenant/create", LOCAL_HOST);

    let req = cli
        .post(path.as_str())
        .json(request)
        .header("x-acc-tenant-id", (*SUPER_TENANT_ID).to_string())
        .header("x-acc-user-id", (*SUPER_USER_ID).to_string())
        .send()
        .await
        .unwrap();
    let d: Uuid = req.json().await.unwrap();
    // Uuid::from_str(&d).unwrap()
    d
}

async fn create_user(request: &CreateUserRequest) -> Uuid {
    let mut cli = reqwest::Client::new();
    let path = format!("{}user/create", LOCAL_HOST);
    let req = cli
        .post(path.as_str())
        .json(request)
        .header("x-acc-tenant-id", (*SUPER_TENANT_ID).to_string())
        .header("x-acc-user-id", (*SUPER_USER_ID).to_string())
        .send()
        .await
        .unwrap();
    let text: String = req.text().await.unwrap();
    println!("received response: {}", text);
    let d: Uuid = serde_json::from_str(&text).unwrap();
    // Uuid::from_str(&d).unwrap()
    d
}

async fn create_currency(
    request: &CreateCurrencyMasterRequest,
    tenant_id: Uuid,
    user_id: Uuid,
) -> Uuid {
    let mut cli = reqwest::Client::new();
    let path = format!("{}currency/create", LOCAL_HOST);
    let req = cli
        .post(path.as_str())
        .json(request)
        .header("x-acc-tenant-id", (tenant_id).to_string())
        .header("x-acc-user-id", (user_id).to_string())
        .send()
        .await
        .unwrap();
    let text: String = req.text().await.unwrap();
    println!("received response: {}", text);
    let d: Uuid = serde_json::from_str(&text).unwrap();
    // Uuid::from_str(&d).unwrap()
    d
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTaxRateRequest {
    pub tax_rate_percentage: f32,
    pub start_date: DateTime<Utc>, //todo ensure that it is not in past more than 24 hours
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCessRequest {
    pub cess_strategy: CessStrategy,
    pub start_date: DateTime<Utc>, //todo ensure that it is not in past more than 24 hours
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
    let mut cli = reqwest::Client::new();
    let path = format!("{}product-item/create", LOCAL_HOST);
    let req = cli
        .post(path.as_str())
        .json(request)
        .header("x-acc-tenant-id", (tenant_id).to_string())
        .header("x-acc-user-id", (user_id).to_string())
        .send()
        .await
        .unwrap();
    let text: String = req.text().await.unwrap();
    println!("received response: {}", text);
    let d: Uuid = serde_json::from_str(&text).unwrap();
    // Uuid::from_str(&d).unwrap()
    d
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvoicingSeriesRequest {
    pub idempotence_key: Uuid,
    pub name: String,
    pub prefix: String,
    pub zero_padded_counter: bool,
    ///primarily for migration purpose and nothing else
    pub start_value: Option<u32>,
    pub financial_year: i32,
}

async fn create_invoicing_series_mst(
    request: &CreateInvoicingSeriesRequest,
    tenant_id: Uuid,
    user_id: Uuid,
) -> Uuid {
    let mut cli = reqwest::Client::new();
    let path = format!("{}invoice-no-series/create", LOCAL_HOST);
    let req = cli
        .post(path.as_str())
        .json(request)
        .header("x-acc-tenant-id", (tenant_id).to_string())
        .header("x-acc-user-id", (user_id).to_string())
        .send()
        .await
        .unwrap();
    let text: String = req.text().await.unwrap();
    println!("received response: {}", text);
    let d: Uuid = serde_json::from_str(&text).unwrap();
    // Uuid::from_str(&d).unwrap()
    d
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBusinessEntityRequest {
    name: String,
    email: String,
    phone: String,
    address_id: Uuid,
    gstin: String,
}

async fn create_business_entity(
    request: &CreateBusinessEntityRequest,
    tenant_id: Uuid,
    user_id: Uuid,
) -> Uuid {
    let mut cli = reqwest::Client::new();
    let path = format!("{}business-entity/create", LOCAL_HOST);
    let req = cli
        .post(path.as_str())
        .json(request)
        .header("x-acc-tenant-id", (tenant_id).to_string())
        .header("x-acc-user-id", (user_id).to_string())
        .send()
        .await
        .unwrap();
    let text: String = req.text().await.unwrap();
    println!("received response: {}", text);
    let d: Uuid = serde_json::from_str(&text).unwrap();
    // Uuid::from_str(&d).unwrap()
    d
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAddressRequest {
    pub idempotence_key: Uuid,
    pub line_1: String,
    pub line_2: Option<String>,
    pub landmark: Option<String>,
    pub city_id: Uuid,
    pub state_id: Uuid,
    pub country_id: Uuid,
    pub pincode_id: Uuid,
}

async fn create_address(request: &CreateAddressRequest, tenant_id: Uuid, user_id: Uuid) -> Uuid {
    let mut cli = reqwest::Client::new();
    let path = format!("{}business-entity/create", LOCAL_HOST);
    let req = cli
        .post(path.as_str())
        .json(request)
        .header("x-acc-tenant-id", (tenant_id).to_string())
        .header("x-acc-user-id", (user_id).to_string())
        .send()
        .await
        .unwrap();
    let text: String = req.text().await.unwrap();
    println!("received response: {}", text);
    let d: Uuid = serde_json::from_str(&text).unwrap();
    // Uuid::from_str(&d).unwrap()
    d
}

#[tokio::main]
async fn main() {
    // let p= reqwest::get("http://localhost:8080/tenant/id/018b33d9-c862-7fde-a0cd-55504d75e5e9")
    //      .await.unwrap();
    //    println!("{}", p.status().as_str());
    //
    //  let pp:Value=p.json().await.unwrap();
    //  println!("Hello, world! {:?}",pp);
    let name = format!("tenant 2 {}", Uuid::now_v7());
    let req = CreateTenantRequest {
        idempotence_key: Uuid::now_v7(),
        display_name: name,
    };
    let tenant_id = get_create_tenant_request(&req).await;
    println!("{}", tenant_id);
    let create_user_req = CreateUserRequest {
        idempotence_key: Uuid::now_v7(),
        tenant_id,
        first_name: "test name 1".to_string(),
        last_name: None,
        email_id: None,
        mobile_number: None,
    };
    let user_id = create_user(&create_user_req).await;
    println!("user_id {}", user_id);
    let create_currency_request = CreateCurrencyMasterRequest {
        idempotence_key: Uuid::now_v7(),
        scale: 2,
        display_name: "INR".to_string(),
        description: "Indian Rupees".to_string(),
    };
    let currency_id = create_currency(&create_currency_request, tenant_id, user_id).await;
    println!("currency id {}", currency_id);
    let product_request = CreateProductRequest {
        idempotence_key: Uuid::now_v7(),
        line_title: format!("some title {}", Uuid::now_v7().as_simple().to_string()),
        line_subtitle: format!("some subtitle {}", Uuid::now_v7().as_simple().to_string()),
        hsn_sac_code: "01013020".to_string(),
        uom: "Piece".to_string(),
        create_tax_request: CreateTaxRateRequest {
            tax_rate_percentage: 12.0,
            start_date: Utc::now(),
        },
        create_cess_request: None,
    };
    let product_id = create_product(&product_request, tenant_id, user_id).await;
    println!("product_id {}", product_id);
    let invoicing_series_req = CreateInvoicingSeriesRequest {
        idempotence_key: Uuid::now_v7(),
        name: "test-series-inv".to_string(),
        prefix: "INV/T1/".to_string(),
        zero_padded_counter: false,
        start_value: None,
        financial_year: 2024,
    };
    let invoicing_series_mst_id =
        create_invoicing_series_mst(&invoicing_series_req, tenant_id, user_id).await;
    println!("invoicing series mst {}", invoicing_series_mst_id);
    let create_address_request = CreateAddressRequest {
        idempotence_key: Uuid::now_v7(),
        line_1: "some address key".to_string(),
        line_2: Some("some address line 2".to_string()),
        landmark: Some("some landmark".to_string()),
        city_id: Uuid::from_str("c7d82fae-7928-7f91-970b-41450b26f197").unwrap(),
        state_id: Uuid::from_str("c42190c1-cc98-7d51-9442-0edebe9b0220").unwrap(),
        country_id: Uuid::from_str("018b05dd-2983-7809-a2d1-95b3f1776eb3").unwrap(),
        pincode_id: Uuid::from_str("c8c1da55-8be8-722c-9623-1295611b2eee").unwrap(),
    };
    let address_id = create_address(&create_address_request, tenant_id, user_id).await;
    println!("address id {}", address_id);
    let create_business_entity_request = CreateBusinessEntityRequest {
        name: "Supplier".to_string(),
        email: "supplier@gmail.com".to_string(),
        phone: "8888888888".to_string(),
        address_id,
        gstin: "07AAAHHHHHHH1Z5".to_string(),
    };
    let business_entity_id =
        create_business_entity(&create_business_entity_request, tenant_id, user_id).await;
    println!("business entity id {}", business_entity_id);
}
