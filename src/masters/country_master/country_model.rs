use uuid::Uuid;
use crate::accounting::currency::currency_models::AuditMetadataBase;


#[derive(Debug)]
pub struct CountryName(String);

impl CountryName{
    pub fn new(name:&str)->Result<CountryName,&'static str>{
        let name = name.trim();
        if name.len()>60{
            return Err("country name cannot be more than 60 chars");
        }
        Ok(CountryName(name.to_ascii_uppercase()))
    }
}
#[derive(Debug)]
pub struct CountryMaster{
    pub id:Uuid,
    pub name:CountryName,//not more than 60 char
    pub audit_metadata:AuditMetadataBase
}



