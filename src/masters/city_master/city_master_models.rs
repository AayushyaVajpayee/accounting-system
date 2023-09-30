use crate::accounting::currency::currency_models::AuditMetadataBase;

struct CityName(String);

impl CityName{
    fn new(name:&str)->Result<Self,&str>{
        let name = name.trim();
        if name.len()>60{
           return Err("cannot have more than 60 chars in city name")
        }
        Ok(Self(name.to_ascii_uppercase()))
    }
}
struct CityMaster{
    id:i32,
    city_name:CityName, //worst case it should not be more than 60
    state_id:i32,
    audit_metadata:AuditMetadataBase
}

