












#[cfg(test)]
pub mod tests{
    use std::str::FromStr;
    use lazy_static::lazy_static;
    use uuid::Uuid;
    lazy_static!{
        pub static ref ADDITIONAL_CHARGE_SEED_ID:Uuid = Uuid::from_str("").unwrap();
    }
}