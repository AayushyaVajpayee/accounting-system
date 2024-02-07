use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;

lazy_static! {
    pub static ref SEED_USER_ID: Uuid =
        Uuid::from_str("018b3444-dc75-7a3f-a4d9-02c41071d3bd").unwrap();
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email_id: Option<String>,
    pub mobile_number: Option<String>,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub idempotence_key: Uuid,
    pub tenant_id: Uuid,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email_id: Option<String>,
    pub mobile_number: Option<String>,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
pub mod tests {
    use uuid::Uuid;

    use crate::accounting::currency::currency_models::{AuditMetadataBase};
    use crate::accounting::user::user_models::{CreateUserRequest, User};
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;
    use crate::accounting::currency::currency_models::tests::an_audit_metadata_base;

    #[derive(Default)]
    pub struct UserTestDataBuilder {
        pub id: Option<Uuid>,
        pub tenant_id: Option<Uuid>,
        pub first_name: Option<String>,
        pub last_name: Option<String>,
        pub email_id: Option<String>,
        pub mobile_number: Option<String>,
        pub audit_metadata: Option<AuditMetadataBase>,
    }


    #[derive(Default)]
    pub struct CreateUserRequestTestBuilder {
        pub idempotence_key: Option<Uuid>,
        pub tenant_id: Option<Uuid>,
        pub first_name: Option<String>,
        pub last_name: Option<String>,
        pub email_id: Option<String>,
        pub mobile_number: Option<String>,
        pub audit_metadata: Option<AuditMetadataBase>,
    }

    pub fn a_user(builder: UserTestDataBuilder) -> User {
        User {
            id: builder.id.unwrap_or_else(Uuid::now_v7),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            first_name: "some user".to_string(),
            last_name: None,
            email_id: None,
            mobile_number: None,
            audit_metadata: an_audit_metadata_base(Default::default()),
        }
    }

    pub fn a_create_user_request(builder: CreateUserRequestTestBuilder) -> CreateUserRequest {
        CreateUserRequest {
            idempotence_key: builder.idempotence_key.unwrap_or_else(Uuid::now_v7),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            first_name: builder.first_name.unwrap_or("".to_string().clone()),
            last_name: builder.last_name,
            email_id: builder
                .email_id
                .or(Some("testemail@t1dno.com".to_string().clone())),
            mobile_number: builder.mobile_number,
            audit_metadata: builder.audit_metadata
                .unwrap_or(an_audit_metadata_base(Default::default())),
        }
    }
}
