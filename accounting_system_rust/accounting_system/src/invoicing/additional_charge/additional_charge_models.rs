use derive_builder::Builder;
use uuid::Uuid;

#[derive(Builder, Debug)]
pub struct CreateAdditionalChargeRequestDbModel {
    pub line_id: Uuid,
    pub line_no: i16,
    pub line_title: String,
    pub title_xx_hash: u32,
    pub rate: f64,
}


#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use lazy_static::lazy_static;
    use uuid::Uuid;
    use xxhash_rust::xxh32;

    use crate::invoicing::additional_charge::additional_charge_models::{CreateAdditionalChargeRequestDbModel, CreateAdditionalChargeRequestDbModelBuilder};

    lazy_static! {
        pub static ref ADDITIONAL_CHARGE_SEED_ID:Uuid = Uuid::from_str("018d557f-4a97-78ef-9947-fcbcebc2be21").unwrap();
    }


pub fn a_create_additional_charge_request_db_model(builder: CreateAdditionalChargeRequestDbModelBuilder) -> CreateAdditionalChargeRequestDbModel {
        let line_title = builder.line_title.unwrap_or("additional charge".to_string());
        let mut k = xxh32::Xxh32::new(0);
        k.update(line_title.as_bytes());
        let a = k.digest();
        CreateAdditionalChargeRequestDbModel {
            line_id: builder.line_id.unwrap_or_else(Uuid::now_v7),
            line_no: builder.line_no.unwrap_or(1),
            line_title,
            title_xx_hash: builder.title_xx_hash.unwrap_or(a),
            rate: builder.rate.unwrap_or(20.0),
        }
    }
}