




#[cfg(test)]
pub mod tests{
    use std::str::FromStr;
    use lazy_static::lazy_static;
    use uuid::Uuid;
    lazy_static!{
        pub static ref SEED_PAYMENT_TERM_ID:Uuid = Uuid::from_str("018d4af9-6bb8-7aad-8a88-86cb15ed88ab").unwrap();
    }
}