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
    pub id: i32,
    pub city_name: CityName, //worst case it should not be more than 60
    pub state_id: i32,
    pub audit_metadata: AuditMetadataBase,
}
