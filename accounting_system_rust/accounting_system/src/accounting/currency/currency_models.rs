use serde::{Deserialize, Serialize};
use derive_builder::Builder;
use uuid::Uuid;
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default,Builder)]
pub struct AuditMetadataBase {
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub created_at: i64,
    pub updated_at: i64,
}
#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone,Builder)]
pub struct CurrencyMaster {
    pub base_master_fields:BaseMasterFields,
    pub scale: i16,
    ///16 char
    pub display_name: String,
    ///50 char
    pub description: String,
    pub audit_metadata: AuditMetadataBase,
}



#[derive(Debug, Serialize, Deserialize, Default,Builder)]
pub struct CreateCurrencyMasterRequest {
    pub idempotence_key: Uuid,
    pub scale: i16,
    pub display_name: String,
    pub description: String,
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
    use crate::accounting::currency::currency_models::{AuditMetadataBase, AuditMetadataBaseBuilder, CreateCurrencyMasterRequest, CreateCurrencyMasterRequestBuilder, CurrencyMaster, CurrencyMasterBuilder};
    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::masters::company_master::company_master_models::base_master_fields::tests::a_base_master_field;


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
            base_master_fields:builder.base_master_fields.unwrap_or_else(||a_base_master_field(Default::default())),
            scale: builder.scale.unwrap_or(0),
            display_name: builder.display_name.unwrap_or("".to_string()),
            description: builder.description.unwrap_or("".to_string()),
            audit_metadata: builder.audit_metadata.unwrap_or_else(|| an_audit_metadata_base(Default::default())),
        }
    }

    pub fn a_create_currency_master_request(builder:
                                            CreateCurrencyMasterRequestBuilder)
                                            -> CreateCurrencyMasterRequest {
        CreateCurrencyMasterRequest {
            idempotence_key: builder.idempotence_key.unwrap_or_else(Uuid::now_v7),
            scale: builder.scale.unwrap_or(0),
            display_name: builder.display_name.unwrap_or("".to_string()),
            description: builder.description.unwrap_or("".to_string()),
        }
    }
}