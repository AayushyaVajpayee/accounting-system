use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;
#[cfg(test)]
use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

lazy_static! {
    pub static ref SEED_ACCOUNT_TYPE_ID:Uuid = Uuid::from_str("7d7ac3ba-ca98-7fac-9881-60f838ea0cd5").unwrap();//todo
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountTypeMaster {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub child_ids: Option<Vec<Uuid>>,
    pub parent_id: Option<Uuid>,
    pub display_name: String,
    pub account_code: Option<i16>,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CreateAccountTypeMasterRequest {
    pub idempotence_key: Uuid,
    pub tenant_id: Uuid,
    pub child_ids: Option<Vec<Uuid>>,
    pub parent_id: Option<Uuid>,
    pub display_name: String,
    pub account_code: Option<i16>,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
#[derive(Debug, Default)]
pub struct CreateAccountTypeMasterRequestTestBuilder {
    pub idempotence_key: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
    pub child_ids: Option<Vec<Uuid>>,
    pub parent_id: Option<Uuid>,
    pub display_name: Option<String>,
    pub account_code: Option<i16>,
    pub audit_metadata: Option<AuditMetadataBase>,
}

#[cfg(test)]
pub fn a_create_account_type_master_request(builder: CreateAccountTypeMasterRequestTestBuilder) -> CreateAccountTypeMasterRequest {
    CreateAccountTypeMasterRequest {
        idempotence_key: builder.idempotence_key.unwrap_or_else(Uuid::now_v7),
        tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
        child_ids: builder.child_ids,
        parent_id: builder.parent_id,
        account_code: builder.account_code,
        display_name: builder.display_name.unwrap_or("".to_string()),
        audit_metadata: builder.audit_metadata.unwrap_or_else(|| crate::accounting::currency::currency_models::an_audit_metadata_base(Default::default())),
    }
}
