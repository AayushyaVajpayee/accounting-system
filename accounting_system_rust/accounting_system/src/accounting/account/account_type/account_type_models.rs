use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;
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

#[derive(Debug, Default, Serialize, Deserialize, Builder)]
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
pub mod tests {
    use crate::accounting::account::account_type::account_type_models::{
        CreateAccountTypeMasterRequest, CreateAccountTypeMasterRequestBuilder,
    };
    use crate::accounting::currency::currency_models::tests::an_audit_metadata_base;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;
    use lazy_static::lazy_static;
    use std::str::FromStr;
    use uuid::Uuid;
    lazy_static! {
        pub static ref SEED_ACCOUNT_TYPE_ID:Uuid = Uuid::from_str("7d7ac3ba-ca98-7fac-9881-60f838ea0cd5").unwrap();//todo
    }
    pub fn a_create_account_type_master_request(
        builder: CreateAccountTypeMasterRequestBuilder,
    ) -> CreateAccountTypeMasterRequest {
        CreateAccountTypeMasterRequest {
            idempotence_key: builder.idempotence_key.unwrap_or_else(Uuid::now_v7),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            child_ids: builder.child_ids.flatten(),
            parent_id: builder.parent_id.flatten(),
            account_code: builder.account_code.flatten(),
            display_name: builder.display_name.unwrap_or("".to_string()),
            audit_metadata: builder
                .audit_metadata
                .unwrap_or_else(|| an_audit_metadata_base(Default::default())),
        }
    }
}
