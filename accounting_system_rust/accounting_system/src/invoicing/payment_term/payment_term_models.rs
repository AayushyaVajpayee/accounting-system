#[cfg(test)]
pub mod tests {
    use std::str::FromStr;
    use std::sync::LazyLock;

    use uuid::Uuid;

    pub static SEED_PAYMENT_TERM_ID: LazyLock<Uuid> =
        LazyLock::new(|| Uuid::from_str("018d4af9-6bb8-7aad-8a88-86cb15ed88ab").unwrap());
}
