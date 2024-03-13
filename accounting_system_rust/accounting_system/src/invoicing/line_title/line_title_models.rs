




#[cfg(test)]
pub mod tests{
    use std::str::FromStr;

    use lazy_static::lazy_static;
    use uuid::Uuid;

    lazy_static!{
        pub static ref SEED_LINE_TITLE_HSN_ID:Uuid = Uuid::from_str("018d4b77-e86b-77f0-ace4-02c34a72399b").unwrap();
    }
}