use std::error::Error;
use std::num::ParseIntError;
use std::pin::Pin;
use crate::accounting::currency::currency_models::AuditMetadataBase;

struct Pincode(u32);

impl Pincode {
    fn new(pincode: &str) -> Result<Self, &str> {
        let code = pincode
            .parse::<u32>()
            .map_err(|_| "cannot parse given pincode as a valid number")?;
        if code<100000 && code>999999{
           return Err("pincode should be 6 digits only")
        }
        Ok(Self(code))
    }
}
struct PincodeMaster {
    id: i32,
    pincode: Pincode,
    city_id: i32,
    audit:AuditMetadataBase,
}
