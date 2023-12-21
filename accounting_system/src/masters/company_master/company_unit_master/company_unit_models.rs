use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::address_master::address_model::{AddressLine, CreateAddressRequest};
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;
use crate::masters::company_master::company_master_models::gstin_no::GstinNo;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct CompanyUnitMaster {
    pub base_master_fields: BaseMasterFields,
    pub company_id: Uuid,
    pub address_id: Uuid,
    pub gstin: GstinNo,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateCompanyUnitRequest {
    pub idempotency_key: Uuid,
    pub tenant_id: Uuid,
    pub company_id: Uuid,
    pub gstin_no: GstinNo,
    // will this be optional or not? also where is the validation
    pub created_by: Uuid,
    pub address: CompanyUnitAddressRequest,
}

impl CreateCompanyUnitRequest {
    pub fn to_create_address_request(&self) -> Option<CreateAddressRequest> {
        match &self.address {
            CompanyUnitAddressRequest::ExistingAddress { .. } => {
                None
            }
            CompanyUnitAddressRequest::NewAddress { request } => {
                Some(CreateAddressRequest {
                    idempotence_key: self.idempotency_key,
                    tenant_id: self.tenant_id,
                    line_1: request.line_1.clone().get_inner(),
                    line_2: request.line_2.clone().map(|a| a.get_inner()),
                    landmark: request.landmark.clone().map(|a| a.get_inner()),
                    city_id: request.city_id,
                    state_id: request.state_id,
                    country_id: request.country_id,
                    pincode_id: request.pincode_id,
                    created_by: self.created_by,
                })
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Builder, Clone, Default)]
pub struct CreateNewCompanyAddressRequest {
    pub line_1: AddressLine,
    pub line_2: Option<AddressLine>,
    pub landmark: Option<AddressLine>,
    pub city_id: Uuid,
    pub state_id: Uuid,
    pub country_id: Uuid,
    pub pincode_id: Uuid,
}


impl From<CreateCompanyUnitRequest> for Option<CreateAddressRequest> {
    fn from(value: CreateCompanyUnitRequest) -> Self {
        match value.address {
            CompanyUnitAddressRequest::ExistingAddress { id } => {
                None
            }
            CompanyUnitAddressRequest::NewAddress { request } => {
                Some(CreateAddressRequest {
                    idempotence_key: value.idempotency_key,
                    tenant_id: value.tenant_id,
                    line_1: request.line_1.get_inner(),
                    line_2: request.line_2.map(|a| a.get_inner()),
                    landmark: request.landmark.map(|a| a.get_inner()),
                    city_id: request.city_id,
                    state_id: request.state_id,
                    country_id: request.country_id,
                    pincode_id: request.pincode_id,
                    created_by: value.created_by,
                })
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CompanyUnitAddressRequest {
    ExistingAddress { id: Uuid },
    NewAddress { request: CreateNewCompanyAddressRequest },
}

#[cfg(test)]
pub mod tests {
    use uuid::Uuid;

    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::masters::address_master::address_model::CreateAddressRequest;
    use crate::masters::address_master::address_model::tests::SEED_ADDRESS_ID;
    use crate::masters::company_master::company_master_models::company_master::tests::SEED_COMPANY_MASTER_ID;
    use crate::masters::company_master::company_master_models::gstin_no::gstin_no_tests::generate_random_gstin_no;
    use crate::masters::company_master::company_unit_master::company_unit_models::{CompanyUnitAddressRequest, CreateCompanyUnitRequest, CreateCompanyUnitRequestBuilder, CreateNewCompanyAddressRequest};
    use crate::tenant::tenant_models::SEED_TENANT_ID;

    impl From<CreateAddressRequest> for CreateNewCompanyAddressRequest {
        fn from(r: CreateAddressRequest) -> Self {
            CreateNewCompanyAddressRequest {
                line_1: r.line_1.try_into().unwrap(),
                line_2: r.line_2.map(|a| a.try_into().unwrap()),
                landmark: r.landmark.map(|a| a.try_into().unwrap()),
                city_id: r.city_id,
                state_id: r.state_id,
                country_id: r.country_id,
                pincode_id: r.pincode_id,
            }
        }
    }

    pub fn a_create_company_unit_request(builder: CreateCompanyUnitRequestBuilder) -> CreateCompanyUnitRequest {
        CreateCompanyUnitRequest {
            idempotency_key: builder.company_id.unwrap_or_else(Uuid::now_v7),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            company_id: builder.company_id.unwrap_or(*SEED_COMPANY_MASTER_ID),
            gstin_no: builder.gstin_no.unwrap_or_else(|| generate_random_gstin_no()
                .get_str()
                .to_string()
                .try_into()
                .unwrap()),
            created_by: builder.created_by.unwrap_or(*SEED_USER_ID),
            address: builder.address.unwrap_or(CompanyUnitAddressRequest::ExistingAddress { id: *SEED_ADDRESS_ID })
        }
    }
}