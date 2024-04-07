use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{generate_random_gstin_no, send_request};
use crate::util::{generate_random_string, generate_random_string_of_numbers};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBusinessEntityRequest {
    idempotence_key: Uuid,
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
    send_request(request, tenant_id, user_id, "business-entity/create").await
}

async fn create_address(request: &CreateAddressRequest, tenant_id: Uuid, user_id: Uuid) -> Uuid {
    send_request(request, tenant_id, user_id, "address/create").await
}


pub async fn create_random_business_entity(tenant_id: Uuid, user_id: Uuid) -> Uuid {
    let idempotence_key = Uuid::now_v7();

    let address = CreateAddressRequest {
        idempotence_key,
        line_1: generate_random_string(20),
        line_2: Some(generate_random_string(20)),
        landmark: Some(generate_random_string(20)),
        city_id: Uuid::from_str("c7d82fae-7928-7f91-970b-41450b26f197").unwrap(),
        state_id: Uuid::from_str("c42190c1-cc98-7d51-9442-0edebe9b0220").unwrap(),
        country_id: Uuid::from_str("018b05dd-2983-7809-a2d1-95b3f1776eb3").unwrap(),
        pincode_id: Uuid::from_str("c8c1da55-8be8-722c-9623-1295611b2eee").unwrap(),
    };
    let address_id = create_address(&address, tenant_id, user_id).await;
    let business_entity = CreateBusinessEntityRequest {
        idempotence_key,
        name: generate_random_string(25),
        email: format!("{}@{}.com", generate_random_string(10), generate_random_string(5)),
        phone: generate_random_string_of_numbers(10),
        address_id,
        gstin: generate_random_gstin_no(),//"29AABCZ2616B1ZK".to_string()
    };
    let id=create_business_entity(&business_entity,tenant_id,user_id).await;
    id
}