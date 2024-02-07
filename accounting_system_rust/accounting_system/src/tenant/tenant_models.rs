use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use derive_builder::Builder;
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;

lazy_static!{
    pub static ref SEED_TENANT_ID:Uuid= Uuid::from_str("018b33d9-c862-7fde-a0cd-55504d75e5e9").unwrap();
}
#[derive(Debug, Serialize, Deserialize, Default, PartialEq,Builder)]
pub struct Tenant {
    pub id: Uuid,
    pub display_name: String,
    pub audit_metadata: AuditMetadataBase,
}


#[derive(Debug, Deserialize, Serialize,Builder)]
pub struct CreateTenantRequest {
    //todo on what basis to uniquely identify tenant?
    // there has to be some business identifier
    // may be a reference number of type gstin etc
    pub idempotence_key:Uuid,
    pub display_name: String,
    pub audit_metadata: AuditMetadataBase,
}






#[cfg(test)]
pub mod tests{
    use uuid::Uuid;
    use crate::tenant::tenant_models::{CreateTenantRequest, CreateTenantRequestBuilder, SEED_TENANT_ID, Tenant, TenantBuilder};
    pub fn a_create_tenant_request(builder:CreateTenantRequestBuilder)->CreateTenantRequest{
        CreateTenantRequest{
            idempotence_key: builder.idempotence_key.unwrap_or_else(Uuid::now_v7),
            display_name: builder.display_name.unwrap_or("".to_string()),
            audit_metadata: builder.audit_metadata.unwrap_or_else(||
                crate::accounting::currency::currency_models::
                an_audit_metadata_base(Default::default())),
        }
    }
    pub fn a_tenant(builder: TenantBuilder) -> Tenant {
        Tenant {
            id: builder.id.unwrap_or(*SEED_TENANT_ID),
            display_name: builder.display_name.unwrap_or("".to_string()),
            audit_metadata: builder.audit_metadata.unwrap_or_else(||
                crate::accounting::currency::currency_models::
                an_audit_metadata_base(Default::default())),
        }
    }
}