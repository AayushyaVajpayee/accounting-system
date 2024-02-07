use serde::{Deserialize, Serialize};
use derive_builder::Builder;
use uuid::Uuid;


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default,Builder)]
pub struct AuditMetadataBase {
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub created_at: i64,
    pub updated_at: i64,
}






#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone,Builder)]
pub struct CurrencyMaster {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub scale: i16,
    ///16 char
    pub display_name: String,
    ///50 char
    pub description: String,
    pub audit_metadata: AuditMetadataBase,
}



#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CreateCurrencyMasterRequest {
    pub idempotence_key: Uuid,
    pub tenant_id: Uuid,
    pub scale: i16,
    pub display_name: String,
    pub description: String,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
#[derive(Debug, Default)]
pub struct CreateCurrencyMasterRequestTestBuilder {
    pub idempotence_key: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
    pub scale: Option<i16>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub audit_metadata: Option<AuditMetadataBase>,
}





#[allow(dead_code)]
#[derive(Debug)]
struct CurrencyAmount {
    scale: i16,
    amount: i64,
}

#[cfg(test)]
pub mod tests{
    use std::str::FromStr;
    use lazy_static::lazy_static;
    use uuid::Uuid;
    use crate::accounting::currency::currency_models::{ AuditMetadataBase, AuditMetadataBaseBuilder, CreateCurrencyMasterRequest, CreateCurrencyMasterRequestTestBuilder, CurrencyMaster, CurrencyMasterBuilder};
    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    lazy_static! {
    pub static ref SEED_CURRENCY_ID:Uuid= Uuid::from_str("018c0bff-4036-7ef8-8383-ae8a38c8ecf1").unwrap();
}

    pub fn an_audit_metadata_base(test_builder: AuditMetadataBaseBuilder) -> AuditMetadataBase {

        AuditMetadataBase {
            created_by: test_builder.created_by.unwrap_or(*SEED_USER_ID),
            updated_by: test_builder.updated_by.unwrap_or(*SEED_USER_ID),
            created_at: test_builder.created_at.unwrap_or(0),
            updated_at: test_builder.updated_at.unwrap_or(0),
        }
    }
    pub fn a_currency_master(builder: CurrencyMasterBuilder) -> CurrencyMaster {
        CurrencyMaster {
            id: builder.id.unwrap_or(Uuid::now_v7()),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            scale: builder.scale.unwrap_or(0),
            display_name: builder.display_name.unwrap_or("".to_string()),
            description: builder.description.unwrap_or("".to_string()),
            audit_metadata: builder.audit_metadata.unwrap_or_else(|| an_audit_metadata_base(Default::default())),
        }
    }

    pub fn a_create_currency_master_request(builder:
                                            CreateCurrencyMasterRequestTestBuilder)
                                            -> CreateCurrencyMasterRequest {
        CreateCurrencyMasterRequest {
            idempotence_key: builder.idempotence_key.unwrap_or_else(Uuid::now_v7),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            scale: builder.scale.unwrap_or(0),
            display_name: builder.display_name.unwrap_or("".to_string()),
            description: builder.description.unwrap_or("".to_string()),
            audit_metadata: builder.audit_metadata
                .unwrap_or_else(||
                    an_audit_metadata_base(Default::default())),
        }
    }
}