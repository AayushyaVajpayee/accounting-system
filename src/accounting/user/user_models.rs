use serde::{Deserialize, Serialize};
use crate::accounting::currency::currency_models::AuditMetadataBase;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub tenant_id: i32,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email_id: Option<String>,
    pub mobile_number: Option<String>,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub tenant_id: i32,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email_id: Option<String>,
    pub mobile_number: Option<String>,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
#[derive(Default)]
pub struct CreateUserRequestTestBuilder {
    pub tenant_id: Option<i32>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email_id: Option<String>,
    pub mobile_number: Option<String>,
    pub audit_metadata: Option<AuditMetadataBase>,
}


#[cfg(test)]
pub fn a_create_user_request(builder: CreateUserRequestTestBuilder) -> CreateUserRequest {
    CreateUserRequest {
        tenant_id: builder.tenant_id.unwrap_or(0),
        first_name: builder.first_name.unwrap_or("".to_string().clone()),
        last_name: builder.last_name,
        email_id: builder.email_id.or(Some("testemail@t1dno.com".to_string().clone())),
        mobile_number: builder.mobile_number,
        audit_metadata: builder.audit_metadata.unwrap_or(crate::accounting::currency::
        currency_models::
        an_audit_metadata_base(Default::default())),
    }
}