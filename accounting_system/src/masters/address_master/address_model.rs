use uuid::Uuid;
use crate::accounting::currency::currency_models::AuditMetadataBase;

#[derive(Debug,Eq, PartialEq)]
pub struct AddressLine(String);


impl AddressLine{
    pub fn new(line:&str)->Result<Self,&str>{
        let line = line.trim();
        if line.len()>60{
            return Err("address line cannot be more than 60 chars")
        } else if line.len()==0{
            return Err("address line cannot be empty")
        }
        return Ok(Self(line.to_string()))
    }
}
#[derive(Debug)]
pub struct Address{
    id:Uuid,
    tenant_id:i32,
    line_1:AddressLine,//Flat, House no., Building, Company, Apartment
    line_2:AddressLine,//Area, Street, Sector, Village
    line_3:Option<AddressLine>,//Landmark
    city_id:i32,
    country_id:Uuid,
    country_specific_fields: CountrySpecificAddressFields,
    audit_metadata:AuditMetadataBase
}


#[derive(Debug)]
pub enum CountrySpecificAddressFields {
    IndiaAddress {
        pincode_id: i32,
        state_id: i32
    }
}




#[cfg(test)]
mod tests{
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;
    use crate::masters::address_master::address_model::AddressLine;

    #[rstest]
    #[case("kldlfdakjfdklajfdlkajafjlkjdalkjflakjdlkajflkajlkjdflkajkldfaj",false,Err("address line cannot be more than 60 chars"))]
    #[case("",false,Err("address line cannot be empty"))]
    #[case("baker street ",true,Ok(AddressLine("baker street".to_string())))]
    fn test_address_line(#[case] input:String, #[case] valid:bool, #[case] output:Result<AddressLine,&'static str>){
        let address_line = AddressLine::new(input.as_str());
        if valid{
            assert_that!(address_line).is_ok();
        }else{
            assert_that!(address_line).is_err();
        }
        assert_eq!(address_line,output);
    }
}