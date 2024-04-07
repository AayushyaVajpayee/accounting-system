use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{SUPER_TENANT_ID, SUPER_USER_ID};
use crate::util::{generate_random_string, generate_random_string_of_numbers, send_request};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub idempotence_key: Uuid,
    pub tenant_id: Uuid,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email_id: Option<String>,
    pub mobile_number: Option<String>,
}
async fn create_user(request: &CreateUserRequest,tenant_id:Uuid,user_id:Uuid) -> Uuid {
    send_request(request, tenant_id, user_id, "user/create").await
}


pub async fn create_random_user_with_super_tenant(tenant_id:Uuid)->Uuid{
    create_random_user(tenant_id,*SUPER_TENANT_ID,*SUPER_USER_ID).await
}

async fn create_random_user(users_tenant_id:Uuid,tenant_id:Uuid,user_id:Uuid)->Uuid{
    let create_user_req = CreateUserRequest {
        idempotence_key: Uuid::now_v7(),
        tenant_id:users_tenant_id,
        first_name: generate_random_string(20),
        last_name:Some(generate_random_string(20)),
        email_id: Some(format!("{}@{}.com",generate_random_string(10),generate_random_string(7))),
        mobile_number: Some(generate_random_string_of_numbers(10)),
    };
    create_user(&create_user_req,tenant_id,user_id).await
}

pub async fn create_random_user_with_other_user_of_same_tenant(tenant_id:Uuid,user_id:Uuid)->Uuid{
    create_random_user(tenant_id,tenant_id,user_id).await
}