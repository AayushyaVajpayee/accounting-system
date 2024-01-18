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
    pub id: Uuid,
    pub state_name: StateName,
    pub audit_metadata: AuditMetadataBase,
    pub country_id: Uuid,
}

#[cfg(test)]
pub mod tests {
    use lazy_static::lazy_static;
    use std::str::FromStr;
    use uuid::Uuid;

    lazy_static! {
        pub static ref SEED_STATE_ID:Uuid= Uuid::from_str("c42190c1-cc98-7d51-9442-0edebe9b0220").unwrap();
    }
}