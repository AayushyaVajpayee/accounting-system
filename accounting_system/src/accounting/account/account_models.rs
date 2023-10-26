use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::accounting::user::user_models::SEED_USER_ID;
use crate::tenant::tenant_models::SEED_TENANT_ID;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Account {
    pub id: i32,
    pub tenant_id: Uuid,
    ///max 20 char string of only numeric data
    pub display_code: String,
    pub account_type_id: i16,
    pub ledger_master_id: i32,
    pub debits_posted: i64,
    pub debits_pending: i64,
    pub credits_posted: i64,
    pub credits_pending: i64,
    pub user_id: Uuid,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    pub tenant_id: Uuid,
    pub display_code: String,
    //todo should it be self generated
    pub account_type_id: i16,
    pub ledger_master_id: i32,
    pub user_id: Uuid,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
#[derive(Debug, Default)]
pub struct CreateAccountRequestTestBuilder {
    pub tenant_id: Option<Uuid>,
    pub display_code: Option<String>,
    pub account_type_id: Option<i16>,
    pub ledger_master_id: Option<i32>,
    pub user_id: Option<Uuid>,
    pub audit_metadata: Option<AuditMetadataBase>,
}

#[cfg(test)]
pub fn a_create_account_request(builder: CreateAccountRequestTestBuilder) -> CreateAccountRequest {
    CreateAccountRequest {
        tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
        display_code: builder.display_code.unwrap_or_else(|| {
            Uuid::now_v7().to_string().split_at(19).0.to_string()
        }),
        account_type_id: builder.account_type_id.unwrap_or(1),
        ledger_master_id: builder.ledger_master_id.unwrap_or(1),
        user_id: builder.user_id.unwrap_or(*SEED_USER_ID),
        audit_metadata: builder.audit_metadata.unwrap_or_else(||
            crate::accounting::currency::currency_models::an_audit_metadata_base(Default::default())),
    }
}