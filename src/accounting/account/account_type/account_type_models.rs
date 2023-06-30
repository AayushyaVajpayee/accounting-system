use serde::{Deserialize, Deserializer};

use crate::accounting::currency::currency_models::AuditMetadataBase;

#[derive(Debug, Deserialize)]
pub struct AccountTypeMaster {
    pub id: i16,
    pub tenant_id: i32,
    pub child_ids: Option<Vec<i16>>,
    pub parent_id: Option<i16>,
    pub display_name: String,
    pub account_code: Option<i16>,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug)]
pub struct CreateAccountTypeMasterRequest {
    pub tenant_id: i32,
    pub child_ids: Option<Vec<i16>>,
    pub parent_id: Option<i16>,
    pub display_name: String,
    pub account_code: Option<i16>,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
#[derive(Debug, Default)]
pub struct CreateAccountTypeMasterRequestTestBuilder {
    pub tenant_id: Option<i32>,
    pub child_ids: Option<Vec<i16>>,
    pub parent_id: Option<i16>,
    pub display_name: Option<String>,
    pub account_code: Option<i16>,
    pub audit_metadata: Option<AuditMetadataBase>,
}

#[cfg(test)]
pub fn a_create_account_type_master_request(builder: CreateAccountTypeMasterRequestTestBuilder) -> CreateAccountTypeMasterRequest {
    CreateAccountTypeMasterRequest {
        tenant_id: builder.tenant_id.unwrap_or(0),
        child_ids: builder.child_ids,
        parent_id: builder.parent_id,
        account_code: builder.account_code,
        display_name: builder.display_name.unwrap_or("".to_string()),
        audit_metadata: builder.audit_metadata.unwrap_or_else(|| crate::accounting::currency::currency_models::an_audit_metadata_base(Default::default())),
    }
}
