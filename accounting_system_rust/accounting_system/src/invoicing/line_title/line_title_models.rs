#[cfg(test)]
pub mod tests {
    use std::str::FromStr;
    use std::sync::LazyLock;

    use uuid::Uuid;

    pub static SEED_LINE_TITLE_HSN_ID: LazyLock<Uuid> =
        LazyLock::new(|| Uuid::from_str("018d4b77-e86b-77f0-ace4-02c34a72399b").unwrap());
}
