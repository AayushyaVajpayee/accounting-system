use anyhow::{ ensure};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;

#[derive(Debug,Serialize, Deserialize, Default, PartialEq)]
pub struct StateName(String);

impl StateName {
    pub fn new(name: &str) -> anyhow::Result<Self> {
        let name = name.trim();
        ensure!(name.len()<=60,"state name cannot be greater than {} chars",60);
        let name = name.to_ascii_uppercase();
        Ok(Self(name))
    }
}

#[derive(Debug,Serialize, Deserialize, Default, PartialEq)]
pub struct StateMasterModel {
    pub id: Uuid,
    pub state_name: StateName,
    ///gst state code
    pub state_code:String,
    pub audit_metadata: AuditMetadataBase,
    pub country_id: Uuid,
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use lazy_static::lazy_static;
    use uuid::Uuid;

    lazy_static! {
        pub static ref SEED_STATE_ID:Uuid= Uuid::from_str("c42190c1-cc98-7d51-9442-0edebe9b0220").unwrap();
    }
}