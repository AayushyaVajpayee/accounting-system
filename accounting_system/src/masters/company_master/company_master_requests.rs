use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug,Serialize,Deserialize)]
pub struct CreateCompanyRequest {
    pub idempotence_key: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub cin: String,
    pub created_by: Uuid,
}

#[cfg(test)]
pub mod tests {
    use crate::masters::company_master::company_master_model::test_data::generate_random_company_identification_number;
    use crate::masters::company_master::company_master_requests::CreateCompanyRequest;
    use crate::tenant::tenant_models::SEED_TENANT_ID;
    use uuid::Uuid;

    #[derive(Debug, Default)]
    pub struct CreateCompanyRequestBuilder {
        pub idempotence_key: Option<Uuid>,
        pub tenant_id: Option<Uuid>,
        pub name: Option<String>,
        pub cin: Option<String>,
        pub created_by: Option<Uuid>,
    }
    pub fn a_create_company_request(builder: CreateCompanyRequestBuilder) -> CreateCompanyRequest {
        CreateCompanyRequest {
            idempotence_key: builder.idempotence_key.unwrap_or_else(Uuid::now_v7),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            name: builder.name.unwrap_or_else(|| "some company".to_string()),
            cin: builder.cin.unwrap_or_else(|| {
                generate_random_company_identification_number()
                    .get_str()
                    .to_string()
            }),
            created_by: Default::default(),
        }
    }
}
