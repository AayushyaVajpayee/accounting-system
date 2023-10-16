use std::str::FromStr;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::tenant::tenant_models::SEED_TENANT_ID;


lazy_static!{
    pub static ref SEED_USER_ID:Uuid= Uuid::from_str("018b3444-dc75-7a3f-a4d9-02c41071d3bd").unwrap();
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
    pub tenant_id: Uuid,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email_id: Option<String>,
    pub mobile_number: Option<String>,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
#[derive(Default)]
pub struct CreateUserRequestTestBuilder {
    pub tenant_id: Option<Uuid>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email_id: Option<String>,
    pub mobile_number: Option<String>,
    pub audit_metadata: Option<AuditMetadataBase>,
}


#[cfg(test)]
pub fn a_create_user_request(builder: CreateUserRequestTestBuilder) -> CreateUserRequest {
    CreateUserRequest {
        tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
        first_name: builder.first_name.unwrap_or("".to_string().clone()),
        last_name: builder.last_name,
        email_id: builder.email_id.or(Some("testemail@t1dno.com".to_string().clone())),
        mobile_number: builder.mobile_number,
        audit_metadata: builder.audit_metadata.unwrap_or(crate::accounting::currency::
        currency_models::
        an_audit_metadata_base(Default::default())),
    }
}