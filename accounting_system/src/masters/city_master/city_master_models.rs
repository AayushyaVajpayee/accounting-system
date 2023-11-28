use uuid::Uuid;
use crate::accounting::currency::currency_models::AuditMetadataBase;

#[derive(Debug)]
pub struct CityName(String);

impl CityName {
    pub fn new(name: &str) -> Result<Self, &'static str> {
        let name = name.trim();
        if name.len() > 60 {
            return Err("cannot have more than 60 chars in city name");
        }
        Ok(Self(name.to_ascii_uppercase()))
    }
}

#[derive(Debug)]
pub struct CityMaster {
    pub id: Uuid,
    pub city_name: CityName, //worst case it should not be more than 60
    pub state_id: Uuid,
    pub audit_metadata: AuditMetadataBase,
    pub country_id:Uuid
}

#[cfg(test)]
pub mod tests {
    use lazy_static::lazy_static;
    use uuid::Uuid;
    use std::str::FromStr;
    lazy_static! {
        pub static ref SEED_CITY_ID:Uuid =  Uuid::from_str("").unwrap();
    }
}