use uuid::Uuid;
use crate::accounting::currency::currency_models::AuditMetadataBase;

#[derive(Debug)]
pub struct Address{
    id:Uuid,
    tenant_id:i32,
    line_1:String,//Flat, House no., Building, Company, Apartment
    line_2:String,//Area, Street, Sector, Village
    line_3:String,//Landmark
    city_id:i32,
    country_id:Uuid,
    additional_fields: AddressAdditionalFields,
    audit_metadata:AuditMetadataBase
}


#[derive(Debug)]
pub enum AddressAdditionalFields {
    IndiaAddress {
        pincode_id: i32,
        state_id: i32
    }
}