#[cfg(test)]
pub mod test_utils_postgres {
    use std::sync::OnceLock;

    use testcontainers::{clients, Container};
    use testcontainers::clients::Cli;
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;

    static K: OnceLock<Cli> = OnceLock::new();

    fn get_client() -> &'static Cli {
        K.get_or_init(|| clients::Cli::default())
    }

    pub fn run_postgres() -> Container<'static, GenericImage> {
        let test_container_client = get_client();
        let image = "postgres";
        let image_tag = "latest";
        let generic_postgres = GenericImage::new(image, image_tag)
            .with_wait_for(WaitFor::message_on_stderr("database system is ready to accept connections"))
            .with_env_var("POSTGRES_DB", "postgres")
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres");
        test_container_client.run(generic_postgres)
    }
}