
use crate::accounting::currency::currency_models::AuditMetadataBase;

#[derive(Debug)]
pub struct Pincode(u32);

impl Pincode {
   pub fn new(pincode: i32) -> Result<Self, &'static str> {
        let code = pincode;
        if code<100000 && code>999999{
           return Err("pincode should be 6 digits only")
        }
        Ok(Self(code as u32))
    }
}
#[derive(Debug)]
pub struct PincodeMaster {
   pub id: i32,
   pub pincode: Pincode,
   pub city_id: i32,
   pub audit_metadata:AuditMetadataBase,
}
