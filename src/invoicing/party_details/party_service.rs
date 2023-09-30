use async_trait::async_trait;

#[async_trait]
trait PartyService{

    async fn register_party();

    async fn get_party_by_id();

    async fn update_party();
}

//how to uniquely identify a party
struct Party{
    id:i32,
    tenant_id:i32,
    name:String, //should be at max 60 char
    address_line_1:String,//should be at max 40 char
    address_line_2:String,//should be at max 40 char
    address_line_3:String,//should be at max 40 char
    event_linking_id:i32,
    pincode:i32,
    state_code:i32,
    city_id:i32,
    country_id:i32,
    gstin:String,//should be valid
}