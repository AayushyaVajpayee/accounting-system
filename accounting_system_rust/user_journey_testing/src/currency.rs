use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::util::{generate_random_string, generate_random_string_of_numbers, send_request};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCurrencyMasterRequest {
    pub idempotence_key: Uuid,
    pub scale: i16,
    pub display_name: String,
    pub description: String,
}

async fn create_currency(
    request: &CreateCurrencyMasterRequest,
    tenant_id: Uuid,
    user_id: Uuid,
) -> Uuid {
    send_request(request, tenant_id, user_id, "currency/create").await
}


pub async fn create_random_currency(
    tenant_id: Uuid, user_id: Uuid,
) -> Uuid {
    let create_currency_request = CreateCurrencyMasterRequest {
        idempotence_key: Uuid::now_v7(),
        scale: 2,
        display_name: format!("CURR{}", generate_random_string_of_numbers(3)),
        description: generate_random_string(20),
    };
    create_currency(&create_currency_request,tenant_id,user_id).await
}