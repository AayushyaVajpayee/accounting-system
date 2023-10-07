use crate::masters::country_master::country_model::{CountryEnum, INDIA_COUNTRY_ID};
use uuid::Uuid;

pub fn get_country_enum_from_id(id: Uuid) -> CountryEnum {
    return if id == *INDIA_COUNTRY_ID {
        CountryEnum::India
    } else {
        CountryEnum::Others
    }
}
