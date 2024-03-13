





#[cfg(test)]
pub mod tests{
    use std::str::FromStr;

    use lazy_static::lazy_static;
    use uuid::Uuid;

    lazy_static!{
        pub static ref SEED_SUBTITLE_ID:Uuid = Uuid::from_str("018d4b83-28ee-7d6f-ab45-fee49bf83ddf").unwrap();
    }
}