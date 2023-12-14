use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;
use crate::masters::company_master::company_master_models::gstin_no::GstinNo;

#[derive(Debug, Serialize, Deserialize,Default,PartialEq)]
pub struct CompanyUnitMaster {
    base_master_fields: BaseMasterFields,
    company_id: Uuid,
    address_id: Uuid,
    gstin: GstinNo,
    audit_metadata: AuditMetadataBase,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateCompanyUnitRequest {
    pub idempotency_key: Uuid,
    pub tenant_id: Uuid,
    pub company_id: Uuid,
    pub address_id: Uuid,
    pub gstin_no: String,
    pub created_by: Uuid,
}


#[cfg(test)]
pub mod tests {
    use uuid::Uuid;

    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::masters::company_master::company_master_models::company_master::tests::SEED_COMPANY_MASTER_ID;
    use crate::masters::company_master::company_master_models::gstin_no::gstin_no_tests::generate_random_gstin_no;
    use crate::masters::company_master::company_unit_master::company_unit_models::{CreateCompanyUnitRequest, CreateCompanyUnitRequestBuilder};
    use crate::tenant::tenant_models::SEED_TENANT_ID;

   pub fn a_create_company_unit_request(builder: CreateCompanyUnitRequestBuilder) -> CreateCompanyUnitRequest {
        CreateCompanyUnitRequest {
            idempotency_key: builder.company_id.unwrap_or_else(Uuid::now_v7),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            company_id: builder.company_id.unwrap_or(*SEED_COMPANY_MASTER_ID),
            address_id: Default::default(),//todo generate address master and its seed
            gstin_no: builder.gstin_no.unwrap_or_else(|| generate_random_gstin_no().get_str().to_string()),
            created_by: builder.created_by.unwrap_or(*SEED_USER_ID),
        }
    }
}