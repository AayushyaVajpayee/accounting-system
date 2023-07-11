#[cfg(test)]
pub mod test_utils_postgres {
    use std::sync::OnceLock;

    use testcontainers::Container;
    use testcontainers::clients::Cli;
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;

    use crate::seeddata::seed_service::copy_tables;

    static K: OnceLock<Cli> = OnceLock::new();
    static PG_CONTAINER: OnceLock<PK<'static>> = OnceLock::new();

    struct PK<'a> {
        container: Container<'a, GenericImage>,
    }

    unsafe impl Send for PK<'_> {}

    unsafe impl Sync for PK<'_> {}

    fn get_client() -> &'static Cli {
        K.get_or_init(|| Cli::default())
    }

    pub fn get_postgres_image_port() -> u16 {
        let k = PG_CONTAINER.get_or_init(|| {
            let container = run_postgres();
            let port = container.get_host_port_ipv4(5432);
            copy_tables(port);
            PK { container }
        }
        );
        k.container.get_host_port_ipv4(5432)
    }

    fn run_postgres() -> Container<'static, GenericImage> {
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