use std::sync::Arc;
use anyhow::{Context, ensure};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::address_master::address_model::{ AddressDto};
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;
use crate::masters::company_master::company_master_models::gstin_no::GstinNo;
#[derive(Debug,Serialize,Deserialize,Default, PartialEq)]
pub struct BusinessEntityDto{
    pub business_entity:BusinessEntityMaster,
    pub address:Option<Arc<AddressDto>>
}
impl BusinessEntityDto{

}
#[derive(Debug, Serialize, Deserialize, Builder, PartialEq, Default)]
pub struct BusinessEntityMaster {
    pub base_master_fields: BaseMasterFields,
    #[serde(flatten)]
    pub entity_type: BusinessEntityType,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum BusinessEntityType {
    EligibleSupplier {
        name: BusinessEntityName,
        email: Email,
        phone: PhoneNumber,
        address_id: Uuid,
        gstin: GstinNo,
    },
    Other {
        name: BusinessEntityName,
        email: Option<Email>,
        phone: PhoneNumber,
        address_id: Option<Uuid>,
        gstin: Option<GstinNo>,
    },
}

impl BusinessEntityType {
    pub fn extract_gstin(&self) -> Option<&GstinNo> {
        match self {
            BusinessEntityType::EligibleSupplier { gstin, .. } => Some(gstin),
            BusinessEntityType::Other { gstin, .. } => gstin.as_ref(),
        }
    }
    pub fn get_name(&self) -> &str {
        match self {
            BusinessEntityType::EligibleSupplier { name, .. } => { name.inner() }
            BusinessEntityType::Other { name, .. } => { name.inner() }
        }
    }

    pub fn get_address_id(&self)->Option<Uuid>{
        match self {
            BusinessEntityType::EligibleSupplier { address_id,.. } => {
                Some(*address_id)
            }
            BusinessEntityType::Other { address_id,.. } => {
                address_id.clone()
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(try_from = "String")]
pub struct BusinessEntityName(String);

impl BusinessEntityName {
    pub fn new(value: &str) -> anyhow::Result<Self> {
        let value = value.trim();
        ensure!(!value.is_empty(),"name cannot be empty");
        ensure!(value.len()<=80,"name cannot be more than 80 chars");
        let valid = value.chars()
            .all(|a| a.is_ascii_alphanumeric() || a == '.' || a == '-' || a == '_' || a == ',' || a == ' ');
        ensure!(valid,"name should have only alphanumeric or chars like '.','-','_',',' ");
        Ok(Self(value.to_string()))
    }
    pub fn inner(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for BusinessEntityName {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        BusinessEntityName::new(value.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(try_from = "String")]
pub struct Email(String);

#[derive(Debug, Validate)]
struct EmailTemp<'a> {
    #[validate(email)] email: &'a str,
}

impl Email {
    pub fn new(value: &str) -> anyhow::Result<Self> {
        let value = value.trim();
        let em = EmailTemp { email: value };
        em.validate().context("email not valid")?;
        Ok(Email(value.to_lowercase()))
    }
    pub fn inner(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for Email {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Email::new(value.as_str())
    }
}

///
/// indian mobile numbers assumed
///
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(try_from = "String")]
pub struct PhoneNumber(String);

impl PhoneNumber {
    pub fn new(value: &str) -> anyhow::Result<Self> {
        let value = value.trim();
        ensure!(!value.is_empty(),"phone number cannot be empty");
        ensure!(value.len()<=10,"phone number too large. check again");
        ensure!(value.chars().all(|a|a.is_numeric()),"indian (+91) mobile numbers (10 digits) allowed only.  \
        Please enter 10 digits without country code and any space ");
        Ok(PhoneNumber(value.to_string()))
    }
    pub fn inner(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for PhoneNumber {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        PhoneNumber::new(value.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateBusinessEntityRequest {
    pub idempotence_key: Uuid,
    pub tenant_id: Uuid,
    #[serde(flatten)]
    pub entity_type: BusinessEntityType,
    pub created_by: Uuid,
}

impl Default for BusinessEntityName {
    fn default() -> Self {
        Self("Default Business Entity".to_string())
    }
}

impl Default for PhoneNumber {
    fn default() -> Self {
        PhoneNumber::new("1234567891").unwrap()
    }
}

impl Default for Email {
    fn default() -> Self {
        Email::new("test@test.test").unwrap()
    }
}

impl Default for BusinessEntityType {
    fn default() -> Self {
        BusinessEntityType::EligibleSupplier {
            name: Default::default(),
            email: Default::default(),
            phone: Default::default(),
            address_id: Default::default(),
            gstin: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;
    use lazy_static::lazy_static;
    use uuid::Uuid;

    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::masters::business_entity_master::business_entity_models::{BusinessEntityMaster, BusinessEntityMasterBuilder, CreateBusinessEntityRequest, CreateBusinessEntityRequestBuilder};
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    lazy_static! {
        pub static ref SEED_BUSINESS_ENTITY_ID1:Uuid = Uuid::from_str("018d5037-bb9d-7263-ba97-d3c46e188c89").unwrap();
    }
    lazy_static! {
        pub static ref SEED_BUSINESS_ENTITY_INVOICE_DTL_ID1:Uuid =Uuid::from_str("018d503d-acef-795b-89ae-dfb0b7feda60").unwrap();
    }
    lazy_static! {
        pub static ref SEED_BUSINESS_ENTITY_ID2:Uuid = Uuid::from_str("018d5efd-009f-7e36-9d4f-8ad30460cada").unwrap();
    }
    lazy_static! {
        pub static ref SEED_BUSINESS_ENTITY_INVOICE_DTL_ID2:Uuid =Uuid::from_str("018d5faf-086c-7347-84a6-cb2b4dcb9dab").unwrap();
    }
    #[allow(dead_code)]
    pub fn a_business_entity_master(b: BusinessEntityMasterBuilder) -> BusinessEntityMaster {
        BusinessEntityMaster {
            base_master_fields: b.base_master_fields.unwrap_or_default(),
            entity_type: b.entity_type.unwrap_or_default(),
            audit_metadata: Default::default(),
        }
    }

    pub fn a_create_business_entity_request(b: CreateBusinessEntityRequestBuilder) -> CreateBusinessEntityRequest {
        CreateBusinessEntityRequest {
            idempotence_key: b.idempotence_key.unwrap_or_else(Uuid::now_v7),
            tenant_id: b.tenant_id.unwrap_or(*SEED_TENANT_ID),
            entity_type: b.entity_type.unwrap_or_default(),
            created_by: b.created_by.unwrap_or(*SEED_USER_ID),
        }
    }
}