use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq,Builder)]
pub struct Tenant {
    pub id: Uuid,
    pub display_name: String,
    pub audit_metadata: AuditMetadataBase,
}


#[derive(Debug, Deserialize, Serialize,Builder)]
pub struct CreateTenantRequest {
    pub idempotence_key:Uuid,
    pub display_name: String,
}






#[cfg(test)]
pub mod tests{
    use std::str::FromStr;

    use lazy_static::lazy_static;
    use uuid::Uuid;

    use crate::accounting::currency::currency_models::tests::an_audit_metadata_base;
    use crate::tenant::tenant_models::{CreateTenantRequest, CreateTenantRequestBuilder, Tenant, TenantBuilder};

    lazy_static!{
    pub static ref SEED_TENANT_ID:Uuid= Uuid::from_str("018b33d9-c862-7fde-a0cd-55504d75e5e9").unwrap();
}
    pub fn a_create_tenant_request(builder:CreateTenantRequestBuilder)->CreateTenantRequest{
        CreateTenantRequest {
            idempotence_key: builder.idempotence_key.unwrap_or_else(Uuid::now_v7),
            display_name: builder.display_name.unwrap_or("".to_string()),
        }
    }
    pub fn a_tenant(builder: TenantBuilder) -> Tenant {
        Tenant {
            id: builder.id.unwrap_or(*SEED_TENANT_ID),
            display_name: builder.display_name.unwrap_or("".to_string()),
            audit_metadata: builder.audit_metadata.unwrap_or_else(||
                an_audit_metadata_base(Default::default())),
        }
    }
}