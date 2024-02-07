use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Account {
    pub id: Uuid,
    pub tenant_id: Uuid,
    ///max 20 char string of only numeric data
    pub display_code: String,
    pub account_type_id: Uuid,
    pub ledger_master_id: Uuid,
    pub debits_posted: i64,
    pub debits_pending: i64,
    pub credits_posted: i64,
    pub credits_pending: i64,
    pub user_id: Uuid,
    //have  forgotten its relevance
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CreateAccountRequest {
    pub idempotence_key: Uuid,
    pub tenant_id: Uuid,
    pub display_code: String,
    //todo should it be self generated
    pub account_type_id: Uuid,
    pub ledger_master_id: Uuid,
    pub user_id: Uuid,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
pub mod tests {
    use lazy_static::lazy_static;
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use std::str::FromStr;
    use uuid::Uuid;

    use crate::accounting::account::account_models::CreateAccountRequest;
    use crate::accounting::account::account_type::account_type_models::tests::SEED_ACCOUNT_TYPE_ID;
    use crate::accounting::currency::currency_models::AuditMetadataBase;
    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::ledger::ledgermaster::ledger_master_models::tests::SEED_LEDGER_MASTER_ID;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    lazy_static! {
        pub static ref SEED_DEBIT_ACCOUNT_ID:Uuid=Uuid::from_str("018c1515-057e-7322-84a7-6f6dc48886d2").unwrap();
    }
    lazy_static! {
        pub static ref SEED_CREDIT_ACCOUNT_ID:Uuid=Uuid::from_str("018c1515-0580-7444-9da8-107986ab3d35").unwrap();
    }

    #[derive(Debug, Default)]
    pub struct CreateAccountRequestTestBuilder {
        pub idempotence_key: Option<Uuid>,
        pub tenant_id: Option<Uuid>,
        pub display_code: Option<String>,
        pub account_type_id: Option<Uuid>,
        pub ledger_master_id: Option<Uuid>,
        pub user_id: Option<Uuid>,
        pub audit_metadata: Option<AuditMetadataBase>,
    }

    pub fn a_create_account_request(builder: CreateAccountRequestTestBuilder) -> CreateAccountRequest {
        CreateAccountRequest {
            idempotence_key: builder.idempotence_key.unwrap_or_else(Uuid::now_v7),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            display_code: builder.display_code.unwrap_or_else(|| {
                let rng = rand::thread_rng();
                rng.sample_iter(Alphanumeric)
                    .take(19)
                    .map(|a| a as char)
                    .collect::<String>()
            }),
            account_type_id: builder.account_type_id.unwrap_or(*SEED_ACCOUNT_TYPE_ID),
            ledger_master_id: builder.ledger_master_id.unwrap_or(*SEED_LEDGER_MASTER_ID),
            user_id: builder.user_id.unwrap_or(*SEED_USER_ID),
            audit_metadata: builder.audit_metadata.unwrap_or_else(||
                crate::accounting::currency::currency_models::an_audit_metadata_base(Default::default())),
        }
    }
}