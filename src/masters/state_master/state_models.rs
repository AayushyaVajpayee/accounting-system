use uuid::Uuid;
use crate::accounting::currency::currency_models::AuditMetadataBase;

#[derive(Debug)]
pub struct StateName(String);

impl StateName {
   pub fn new(name: &str) -> Result<Self, &'static str> {
        let name = name.trim();
        if name.len() > 60 {
            return Err("state name cannot be greater than 60 chars");
        }
        let name = name.to_ascii_uppercase();
        Ok(Self(name))
    }
}

#[derive(Debug)]
pub struct StateMasterModel {
    pub id: i32,
    pub state_name: StateName,
    pub audit_metadata:AuditMetadataBase,
    pub country_id:Uuid
}
