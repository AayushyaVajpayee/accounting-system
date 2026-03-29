#[cfg(test)]
pub mod tests {
    use std::str::FromStr;
    use std::sync::LazyLock;

    use uuid::Uuid;

    pub static SEED_SUBTITLE_ID: LazyLock<Uuid> =
        LazyLock::new(|| Uuid::from_str("018d4b83-28ee-7d6f-ab45-fee49bf83ddf").unwrap());
}
