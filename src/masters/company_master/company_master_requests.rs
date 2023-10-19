use uuid::Uuid;

#[derive(Debug)]
pub struct CreateCompanyRequest {
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
        pub tenant_id: Option<Uuid>,
        pub name: Option<String>,
        pub cin: Option<String>,
        pub created_by: Option<Uuid>,
    }
    pub fn a_create_company_request(builder: CreateCompanyRequestBuilder) -> CreateCompanyRequest {
        CreateCompanyRequest {
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
