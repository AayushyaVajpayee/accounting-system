use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{SUPER_TENANT_ID, SUPER_USER_ID};
use crate::util::{generate_random_string, send_request};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTenantRequest {
    idempotence_key: Uuid,
    display_name: String,
}

async fn get_create_tenant_request(request: &CreateTenantRequest) -> Uuid {
    send_request(request, *SUPER_TENANT_ID, *SUPER_USER_ID, "tenant/create").await
}

pub async fn create_random_tenant() -> Uuid {
    let po = CreateTenantRequest {
        idempotence_key: Uuid::now_v7(),
        display_name: generate_random_string(23),
    };
    get_create_tenant_request(&po).await
}