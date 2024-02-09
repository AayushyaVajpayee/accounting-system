use anyhow::bail;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct AddressLine(String);

impl Default for AddressLine {
    fn default() -> Self {
        AddressLine("some fake address".to_string())
    }
}
impl AddressLine {
    pub fn new_nullable(line: Option<&str>) -> anyhow::Result<Option<Self>> {
        match line {
            None => { Ok(None) }
            Some(line) => {
                Some(AddressLine::new(line)).transpose()
            }
        }
    }
    pub fn new(line: &str) -> anyhow::Result<Self> {
        let line = line.trim();
        if line.len() > 60 {
            bail!("address line cannot be more than 60 chars");
        } else if line.is_empty() {
            bail!("address line cannot be empty");
        }
        Ok(Self(line.to_string()))
    }
    pub fn get_inner(self) -> String {
        self.0
    }
}

impl TryFrom<String> for AddressLine {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim();
        AddressLine::new(value)
    }
}


#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Address {
    pub base_master_fields: BaseMasterFields,
    pub line_1: AddressLine,
    //Flat, House no., Building, Company, Apartment
    pub line_2: Option<AddressLine>,
    //Area, Street, Sector, Village
    pub landmark: Option<AddressLine>,
    //Landmark
    pub city_id: Uuid,
    pub state_id: Uuid,
    pub pincode_id: Uuid,
    pub country_id: Uuid,
    pub audit_metadata: AuditMetadataBase,
}


#[derive(Debug, Serialize, Deserialize, Builder, Clone)]
pub struct CreateAddressRequest {
    pub idempotence_key: Uuid,
    pub tenant_id: Uuid,
    pub line_1: String,
    pub line_2: Option<String>,
    pub landmark: Option<String>,
    pub city_id: Uuid,
    pub state_id: Uuid,
    pub country_id: Uuid,
    pub pincode_id: Uuid,
    pub created_by: Uuid

}


#[cfg(test)]
pub mod tests {
    use anyhow::anyhow;
    use lazy_static::lazy_static;
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;
    use std::str::FromStr;
    use uuid::Uuid;

    use crate::accounting::currency::currency_models::tests::an_audit_metadata_base;
    use crate::accounting::currency::currency_models:: AuditMetadataBase;
    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::masters::address_master::address_model::{Address, AddressLine, CreateAddressRequest, CreateAddressRequestBuilder};
    use crate::masters::city_master::city_master_models::tests::SEED_CITY_ID;
    use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;
    use crate::masters::company_master::company_master_models::base_master_fields::tests::a_base_master_field;
    use crate::masters::country_master::country_model::INDIA_COUNTRY_ID;
    use crate::masters::pincode_master::pincode_models::tests::SEED_PINCODE_ID;
    use crate::masters::state_master::state_models::tests::SEED_STATE_ID;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    lazy_static! {
        pub static ref SEED_ADDRESS_ID:Uuid = Uuid::from_str("018c6261-186b-763f-a3ae-13d44e2bf01d").unwrap();
    }
    #[allow(dead_code)]
    pub struct AddressBuilder {
        base_master_fields: Option<BaseMasterFields>,
        line_1: Option<AddressLine>,
        line_2: Option<AddressLine>,
        landmark: Option<AddressLine>,
        //Landmark
        city_id: Option<Uuid>,
        state_id: Option<Uuid>,
        pincode_id: Option<Uuid>,
        country_id: Option<Uuid>,
        audit_metadata: Option<AuditMetadataBase>,
    }

    pub fn an_address(builder: AddressBuilder) -> Address {
        Address {
            base_master_fields: builder.base_master_fields.unwrap_or_else(|| a_base_master_field(Default::default())),
            line_1: AddressLine("some fake address".to_string()),
            line_2: None,
            landmark: None,
            city_id: builder.city_id.unwrap_or(*SEED_CITY_ID),
            state_id: builder.state_id.unwrap_or(*SEED_STATE_ID),
            pincode_id: builder.pincode_id.unwrap_or(*SEED_PINCODE_ID),
            country_id: builder.country_id.unwrap_or(*INDIA_COUNTRY_ID),
            audit_metadata: builder.audit_metadata.unwrap_or_else(|| an_audit_metadata_base(Default::default())),
        }
    }

    pub fn a_create_address_request(builder: CreateAddressRequestBuilder) -> CreateAddressRequest {
        CreateAddressRequest {
            idempotence_key: builder.idempotence_key.unwrap_or_else(Uuid::now_v7),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            line_1: builder.line_1.unwrap_or("some fake address".to_string()),
            line_2: None,
            landmark: None,
            city_id: builder.city_id.unwrap_or(*SEED_CITY_ID),
            state_id: builder.state_id.unwrap_or(*SEED_STATE_ID),
            country_id: builder.country_id.unwrap_or(*INDIA_COUNTRY_ID),
            pincode_id: builder.pincode_id.unwrap_or(*SEED_PINCODE_ID),
            created_by: builder.created_by.unwrap_or(*SEED_USER_ID)
        }
    }

    #[rstest]
    #[case("kldlfdakjfdklajfdlkajafjlkjdalkjflakjdlkajflkajlkjdflkajkldfaj", false, Err(anyhow ! ("address line cannot be more than 60 chars")))]
    #[case("", false, Err(anyhow ! ("address line cannot be empty")))]
    #[case("baker street ", true, Ok(AddressLine("baker street".to_string())))]
    fn test_address_line(#[case] input: String, #[case] valid: bool, #[case] output: anyhow::Result<AddressLine>) {
        let address_line = AddressLine::new(input.as_str());
        if valid {
            assert_that!(address_line).is_ok();
            assert_that!(address_line.unwrap()).is_equal_to(output.unwrap())
        } else {
            assert_that!(address_line).is_err();
            assert_that!(address_line.unwrap_err().to_string())
                .is_equal_to(output.unwrap_err().to_string());
        }
    }
}