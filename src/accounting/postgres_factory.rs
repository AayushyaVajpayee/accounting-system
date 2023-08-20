use std::sync::OnceLock;
use std::time::Duration;

use deadpool_postgres::{ManagerConfig, Pool, RecyclingMethod, Runtime};
use deadpool_postgres::RecyclingMethod::{Clean, Fast, Verified};
use tokio_postgres::{Config, NoTls};

use crate::configurations::{get_dev_conf, Setting};

static CONNECTION_POOL: OnceLock<Pool> = OnceLock::new();

pub fn get_postgres_conn_pool() -> &'static Pool {
    CONNECTION_POOL.get_or_init(init)
}

fn init() -> Pool {
    let settings: Setting = get_dev_conf();
    let cfg = get_pg_config(&settings);
    let mgr = deadpool_postgres::Manager::from_config(cfg, NoTls, ManagerConfig {
        recycling_method: get_recycling_method(settings.db.recycling_method.as_str())
    });
    Pool::builder(mgr)
        .max_size(settings.db.max_connections as usize)
        .runtime(Runtime::Tokio1)
        .create_timeout(Some(Duration::from_secs(settings.db.connect_timeout_seconds as u64)))
        .build().unwrap()
}

fn get_pg_config(settings: &Setting) -> Config {
    let mut cfg = Config::new();
    cfg.host(settings.db.host.as_str());
    cfg.port(settings.db.port);
    cfg.user(settings.db.user.as_str());
    cfg.password(settings.db.password.as_str());
    cfg.dbname(settings.db.db.as_str());
    cfg.application_name(settings.db.application_name.as_str());
    cfg.connect_timeout(Duration::from_secs(settings.db.connect_timeout_seconds as u64));
    cfg
}

fn get_recycling_method(recycling_method_str: &str) -> RecyclingMethod {
    match recycling_method_str {
        "Fast" => Fast,
        "Clean" => Clean,
        "Verified" => Verified,
        _ => panic!("{} not supported as a recycling method", recycling_method_str)
    }
}

#[cfg(test)]
pub mod test_utils_postgres {
    use std::time::Duration;

    use deadpool_postgres::{ManagerConfig, Pool, Runtime};
    use testcontainers::clients::Cli;
    use testcontainers::Container;
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;
    use tokio::sync::OnceCell;
    use tokio_postgres::{Config, NoTls};

    use crate::accounting::postgres_factory::get_recycling_method;
    use crate::configurations::configuration_test_code::{get_tests_conf, TestSettings};
    use crate::seeddata::seed_service::get_seed_service_with_pool_supplied;

    static CONNECTION_POOL: OnceCell<Pool> = OnceCell::const_new();
    static TEST_CONTAINER_CLIENT: OnceCell<Cli> = OnceCell::const_new();
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
        let settings: TestSettings = get_tests_conf();
        let cfg = get_pg_config(&settings, port);
        let mgr = deadpool_postgres::Manager::from_config(cfg, NoTls,
                                                          ManagerConfig {
                                                              recycling_method:
                                                              get_recycling_method(
                                                                  settings.db
                                                                      .recycling_method
                                                                      .as_str())
                                                          });
        Pool::builder(mgr)
            .max_size(settings.db.max_connections as usize)
            .runtime(Runtime::Tokio1)
            .create_timeout(Some(Duration::from_secs(settings.db.connect_timeout_seconds as u64)))
            .wait_timeout(Some(Duration::from_secs(settings.db.wait_timeout_seconds as u64)))
            .build().unwrap()
    }


    async fn get_client() -> &'static Cli {
        TEST_CONTAINER_CLIENT.get_or_init(init_cli).await
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
        let pool = get_postgres_conn_pool(port).await;
        let seed_s = get_seed_service_with_pool_supplied(pool);
        seed_s.copy_tables().await;
        PK { container }
    }

    async fn run_postgres() -> Container<'static, GenericImage> {
        let settings: TestSettings = get_tests_conf();
        let test_container_client = get_client().await;
        let image = settings.docker_postgres_detail.image;
        let image_tag = settings.docker_postgres_detail.image_tag;
        let generic_postgres = GenericImage::new(image, image_tag)
            .with_wait_for(WaitFor::message_on_stderr("database system is ready to accept connections"))
            .with_env_var("POSTGRES_DB", settings.db.db)
            .with_env_var("POSTGRES_USER", settings.db.user)
            .with_env_var("POSTGRES_PASSWORD", settings.db.password);
        test_container_client.run(generic_postgres)
    }

    fn get_pg_config(settings: &TestSettings, port: u16) -> Config {
        let mut cfg = Config::new();
        cfg.host(settings.db.host.as_str());
        cfg.port(port);
        cfg.user(settings.db.user.as_str());
        cfg.password(settings.db.password.as_str());
        cfg.dbname(settings.db.db.as_str());
        cfg.application_name(settings.db.application_name.as_str());
        cfg.connect_timeout(Duration::from_secs(settings.db.connect_timeout_seconds as u64));
        cfg
    }
}
