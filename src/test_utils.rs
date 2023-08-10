#[cfg(test)]
pub mod test_utils_postgres {
    use std::time::Duration;

    use deadpool_postgres::{ManagerConfig, Pool, Runtime};
    use deadpool_postgres::RecyclingMethod::{Clean, Fast, Verified};
    use postgres::Client;
    use testcontainers::clients::Cli;
    use testcontainers::{Container, Image};
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;
    use tokio::sync::OnceCell;
    use tokio_postgres::{Config, NoTls};

    use crate::seeddata::seed_service::copy_tables;

    static CONNECTION_POOL: OnceCell<Pool> = OnceCell::const_new();
    static K: OnceCell<Cli> = OnceCell::const_new();
    static PG_CONTAINER: OnceCell<PK<'static>> = OnceCell::const_new();

    struct PK<'a> {
        container: Container<'a, GenericImage>,
    }

    unsafe impl Send for PK<'_> {}

    unsafe impl Sync for PK<'_> {}

    pub async fn get_postgres_conn_pool(port: u16) -> &'static Pool {
        CONNECTION_POOL.get_or_init(|| init_pool(port)).await
    }

    async fn init_pool(port: u16) -> Pool {
        let mut cfg = Config::new();
        cfg.host("localhost");
        cfg.port(port);
        cfg.user("postgres");
        cfg.password("postgres");
        cfg.dbname("postgres");
        cfg.application_name("accounting-system");
        // cfg.connect_timeout(Duration::from_secs(15));
        let mgr = deadpool_postgres::Manager::from_config(cfg, NoTls, ManagerConfig { recycling_method: Clean });
        Pool::builder(mgr)
            .max_size(1)
            // .runtime(Runtime::Tokio1)
            // .create_timeout(Some(Duration::from_secs(10)))
            .build().unwrap()
    }
    pub fn create_postgres_client(port: u16) -> Client {
        let mut cfg = Config::new();
        cfg.host("localhost");
        cfg.port(port);
        cfg.user("postgres");
        cfg.password("postgres");
        cfg.dbname("postgres");
        cfg.application_name("accounting-system");
        cfg.connect_timeout(Duration::from_secs(20));
        let mut mgr = deadpool_postgres::Manager::from_config(cfg, NoTls, ManagerConfig { recycling_method: Fast });
        let connection_pool1 = deadpool_postgres::Pool::builder(mgr)
            .max_size(1)
            .runtime(Runtime::Tokio1)
            .create_timeout(Some(Duration::from_secs(20)))
            .build().unwrap();

        let con_str =
            format!("host=localhost user=postgres password=postgres dbname=postgres port={port}");
        let client = Client::
        connect(&con_str, NoTls)
            .unwrap();
        client
    }

    async fn get_client() -> &'static Cli {
        K.get_or_init(init_cli).await
    }

    async fn init_cli() -> Cli {
        Cli::default()
    }

    pub async fn get_postgres_image_port() -> u16 {
        let k = PG_CONTAINER.get_or_init(init_container).await;
        k.container.get_host_port_ipv4(5432)
    }

    async fn init_container() -> PK<'static> {
        let container = run_postgres().await;
        let port = container.get_host_port_ipv4(5432);
        println!("copy tables starting");

        copy_tables(port).await;
        println!("copy tables completed");
        PK { container }
    }

    async fn run_postgres() -> Container<'static, GenericImage> {
        let test_container_client = get_client().await;
        let image = "postgres";
        let image_tag = "latest";
        let mut generic_postgres = GenericImage::new(image, image_tag)
            .with_wait_for(WaitFor::message_on_stderr("database system is ready to accept connections"))
            .with_env_var("POSTGRES_DB", "postgres")
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres");
        test_container_client.run(generic_postgres)
    }
}