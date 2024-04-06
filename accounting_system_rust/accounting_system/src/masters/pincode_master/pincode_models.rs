use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::country_master::country_model::CountryEnum;
use crate::masters::country_master::country_utils::get_country_enum_from_id;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct PincodeMaster {
    pub id: Uuid,
    pub pincode: Pincode,
    pub city_id: Uuid,
    pub audit_metadata: AuditMetadataBase,
    pub country_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Pincode {
    IndianPincode(u32),
    Others(String),
}
impl Default for Pincode {
    fn default() -> Self {
        Pincode::IndianPincode(249407)
    }
}
impl Pincode {
    pub fn new(pincode: &str, country_id: Uuid) -> Result<Self, &'static str> {
        let country_enum = get_country_enum_from_id(country_id);
        let pincode = pincode.trim();
        match country_enum {
            CountryEnum::India => {
                let pincode = pincode
                    .parse::<u32>()
                    .map_err(|_| "india pincode has to be numeric")?;
                if !(100000..=999999).contains(&pincode) {
                    return Err("pincode should be 6 digits only");
                }
                Ok(Pincode::IndianPincode(pincode))
            }
            CountryEnum::Others => {
                if pincode.len() > 20 {
                    return Err("pincode length cannot be greater than 20 for non indian pincodes");
                }
                Ok(Pincode::Others(pincode.to_string()))
            }
        }
    }

    pub fn to_string(&self) -> String {
        match &self {
            Pincode::IndianPincode(a) => a.to_string(),
            Pincode::Others(a) => a.to_string(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use lazy_static::lazy_static;
    use rstest::rstest;
    use speculoos::assert_that;
    use speculoos::prelude::ResultAssertions;
    use uuid::Uuid;

    use crate::masters::country_master::country_model::INDIA_COUNTRY_ID;
    use crate::masters::pincode_master::pincode_models::Pincode;

    lazy_static! {
        pub static ref SEED_PINCODE_ID: Uuid =
            Uuid::from_str("c8c1da55-8be8-722c-9623-1295611b2eee").unwrap();
    }

    #[rstest]
    #[case("0000c0",false,Err("india pincode has to be numeric"),*INDIA_COUNTRY_ID)]
    #[case("123456",true,Ok(Pincode::IndianPincode(123456)),*INDIA_COUNTRY_ID)]
    #[case("9999999",false,Err("pincode should be 6 digits only"),*INDIA_COUNTRY_ID)]
    #[case("1234567",true,Ok(Pincode::Others("1234567".to_string())),Uuid::now_v7())]
    #[case(
        "123456789123bhudhcui38kj",
        false,
        Err("pincode length cannot be greater than 20 for non indian pincodes"),
        Uuid::now_v7()
    )]
    fn test_pincodes(
        #[case] input: String,
        #[case] valid: bool,
        #[case] output: Result<Pincode, &'static str>,
        #[case] country_id: Uuid,
    ) {
        let p = Pincode::new(input.as_str(), country_id);
        if valid {
            assert_that!(p).is_ok();
            assert_eq!(p, output);
        } else {
            assert_that!(p).is_err();
            assert_eq!(p, output);
        }
    }
}
