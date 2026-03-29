use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;

pub static INDIA_COUNTRY_ID: std::sync::LazyLock<Uuid> =
    std::sync::LazyLock::new(|| Uuid::from_str("018b05dd-2983-7809-a2d1-95b3f1776eb3").unwrap());
#[derive(Debug)]
pub enum CountryEnum {
    India,
    Others,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct CountryName(String);

impl CountryName {
    pub fn new(name: &str) -> Result<CountryName, &'static str> {
        let name = name.trim();
        if name.len() > 60 {
            return Err("country name cannot be more than 60 chars");
        }
        Ok(CountryName(name.to_ascii_uppercase()))
    }
}
#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct CountryMaster {
    pub id: Uuid,
    pub name: CountryName, //not more than 60 char
    pub audit_metadata: AuditMetadataBase,
}
