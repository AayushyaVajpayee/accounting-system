use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::country_master::country_model::CountryEnum;
use crate::masters::country_master::country_utils::get_country_enum_from_id;
use uuid::Uuid;

#[derive(Debug)]
pub struct PincodeMaster {
    pub id: i32,
    pub pincode: Pincode,
    pub city_id: i32,
    pub audit_metadata: AuditMetadataBase,
    pub country_id:Uuid
}

#[derive(Debug,PartialEq,Eq)]
pub enum Pincode {
    IndianPincode(u32),
    Others(String),
}

impl Pincode {
    pub fn new(pincode: &str, country_id: Uuid) -> Result<Self, &'static str> {
        let country_enum = get_country_enum_from_id(country_id);
        let pincode = pincode.trim();
        match country_enum {
            CountryEnum::India => {
                let pincode = pincode
                    .parse::<u32>()
                    .map_err(|a| "india pincode has to be numeric")?;
                if !(100000..=999999).contains(&pincode) {
                    return Err("pincode should be 6 digits only");
                }
                Ok(Pincode::IndianPincode(pincode))
            }
            CountryEnum::Others =>
                {
                    if pincode.len()>20{
                        return Err("pincode length cannot be greater than 20 for non indian pincodes")
                    }
                    Ok(Pincode::Others(pincode.to_string()))
                },
        }
    }
}


#[cfg(test)]
mod tests{
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;
    use uuid::Uuid;
    use crate::masters::pincode_master::pincode_models::Pincode;
    use crate::masters::country_master::country_model::INDIA_COUNTRY_ID;

    #[rstest]
    #[case("0000c0",false,Err("india pincode has to be numeric"),*INDIA_COUNTRY_ID)]
    #[case("123456",true,Ok(Pincode::IndianPincode(123456)),*INDIA_COUNTRY_ID)]
    #[case("9999999",false,Err("pincode should be 6 digits only"),*INDIA_COUNTRY_ID)]
    #[case("1234567",true,Ok(Pincode::Others("1234567".to_string())),Uuid::now_v7())]
    #[case("123456789123bhudhcui38kj",false,Err("pincode length cannot be greater than 20 for non indian pincodes"),Uuid::now_v7())]
    fn test_pincodes(#[case] input:String,#[case] valid:bool, #[case] output: Result<Pincode,&'static str>,#[case] country_id:Uuid){
        let p = Pincode::new(input.as_str(),country_id);
        if valid{
            assert_that!(p).is_ok();
            assert_eq!(p,output);
        }else{
            assert_that!(p).is_err();
            assert_eq!(p,output);
        }

    }
}