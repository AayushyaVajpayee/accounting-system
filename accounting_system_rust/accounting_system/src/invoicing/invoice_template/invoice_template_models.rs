


#[cfg(test)]
pub mod tests{
    use std::str::FromStr;
    use lazy_static::lazy_static;
    use uuid::Uuid;
    lazy_static!{
        pub static ref SEED_INVOICE_TEMPLATE_ID:Uuid =Uuid::from_str("018d5552-fb70-7d28-bbf6-7e726e5c15eb").unwrap();
    }
}