use deadpool_postgres::RecyclingMethod::{Clean, Fast, Verified};
use deadpool_postgres::{ManagerConfig, Pool, RecyclingMethod, Runtime};
use std::time::Duration;
use tokio_postgres::{Config, NoTls};

use crate::configurations::{get_dev_conf, Setting};

pub fn get_postgres_conn_pool() -> Pool {
    init()
}

fn init() -> Pool {
    let settings: Setting = get_dev_conf();
    let cfg = get_pg_config(&settings);
    let mgr = deadpool_postgres::Manager::from_config(
        cfg,
        NoTls,
        ManagerConfig {
            recycling_method: get_recycling_method(settings.db.recycling_method.as_str()),
        },
    );
    Pool::builder(mgr)
        .max_size(settings.db.max_connections as usize)
        .runtime(Runtime::Tokio1)
        .create_timeout(Some(Duration::from_secs(
            settings.db.connect_timeout_seconds as u64,
        )))
        .build()
        .unwrap()
}

fn get_pg_config(settings: &Setting) -> Config {
    let mut cfg = Config::new();
    cfg.host(settings.db.host.as_str());
    cfg.port(settings.db.port);
    cfg.user(settings.db.user.as_str());
    cfg.password(settings.db.password.as_str());
    cfg.dbname(settings.db.db.as_str());
    cfg.application_name(settings.db.application_name.as_str());
    cfg.connect_timeout(Duration::from_secs(
        settings.db.connect_timeout_seconds as u64,
    ));
    cfg
}

fn get_recycling_method(recycling_method_str: &str) -> RecyclingMethod {
    match recycling_method_str {
        "Fast" => Fast,
        "Clean" => Clean,
        "Verified" => Verified,
        _ => panic!(
            "{} not supported as a recycling method",
            recycling_method_str
        ),
    }
}

#[cfg(test)]
pub mod test_utils_postgres {
    use deadpool_postgres::{Manager, ManagerConfig, Pool, Runtime};
    use std::sync::Arc;
    use std::time::Duration;
    use testcontainers::core::WaitFor;
    use testcontainers::runners::AsyncRunner;
    use testcontainers::{ContainerAsync, GenericImage, ImageExt};
    use tokio::sync::OnceCell;
    use tokio_postgres::{Config, NoTls};

    use crate::accounting::postgres_factory::get_recycling_method;
    use crate::configurations::{get_dev_conf, Setting};
    use crate::db_schema_syncer::db_struct_mapper::init_db_with_seed;

    static PG_CONTAINER: OnceCell<ContainerAsync<GenericImage>> = OnceCell::const_new();

    pub async fn get_dao_generic<T, F>(f: F, dbname: Option<&str>) -> T
    where
        F: FnOnce(Arc<Pool>) -> T,
    {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, dbname).await;
        f(postgres_client)
    }

    pub async fn get_postgres_conn_pool(port: u16, dbname: Option<&str>) -> Arc<Pool> {
        init_pool(port, dbname).await
    }

    pub async fn get_postgres_conn_pool_with_new_db(port: u16, dbname: &str) -> Arc<Pool> {
        let pool = init_pool(port, Some(dbname)).await;
        init_db_with_seed(pool.clone()).await;
        pool
    }

    fn build_pool(mgr: Manager, settings: &Setting) -> Pool {
        Pool::builder(mgr)
            .max_size(1)
            .runtime(Runtime::Tokio1)
            .create_timeout(Some(Duration::from_secs(
                settings.db.connect_timeout_seconds as u64,
            )))
            .wait_timeout(Some(Duration::from_secs(
                settings.db.wait_timeout_seconds as u64,
            )))
            .build()
            .unwrap()
    }

    async fn init_pool(port: u16, dbname: Option<&str>) -> Arc<Pool> {
        let settings: Setting = get_dev_conf();
        let mut cfg = get_pg_config(&settings, port);
        let mut to_be_seeded = false;
        if let Some(dbname) = dbname {
            let mgr = Manager::from_config(
                cfg.clone(),
                NoTls,
                ManagerConfig {
                    recycling_method: get_recycling_method(settings.db.recycling_method.as_str()),
                },
            );

            let p = build_pool(mgr, &settings);
            let k = format!("create database {};", dbname);
            p.get()
                .await
                .unwrap()
                .simple_query(k.as_str())
                .await
                .unwrap();
            cfg.dbname(dbname);
            to_be_seeded = true;
        }

        let mgr = Manager::from_config(
            cfg,
            NoTls,
            ManagerConfig {
                recycling_method: get_recycling_method(settings.db.recycling_method.as_str()),
            },
        );
        let p = Arc::new(build_pool(mgr, &settings));
        if to_be_seeded {
            init_db_with_seed(p.clone()).await;
        }
        p
    }

    pub async fn get_postgres_image_port() -> u16 {
        let container = PG_CONTAINER.get_or_init(init_container).await;
        container.get_host_port_ipv4(5432).await.unwrap()
    }

    async fn init_container() -> ContainerAsync<GenericImage> {
        let container = run_postgres().await;
        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let pool = get_postgres_conn_pool(port, None).await;
        init_db_with_seed(pool).await;
        container
    }

    async fn run_postgres() -> ContainerAsync<GenericImage> {
        let settings: Setting = get_dev_conf();
        GenericImage::new("postgres", "16.6")
            .with_wait_for(WaitFor::message_on_stderr(
                "database system is ready to accept connections",
            ))
            .with_env_var("POSTGRES_DB", settings.db.db)
            .with_env_var("POSTGRES_USER", settings.db.user)
            .with_env_var("POSTGRES_PASSWORD", settings.db.password)
            .start()
            .await
            .unwrap()
    }

    fn get_pg_config(settings: &Setting, port: u16) -> Config {
        let mut cfg = Config::new();
        cfg.host(settings.db.host.as_str());
        cfg.port(port);
        cfg.user(settings.db.user.as_str());
        cfg.password(settings.db.password.as_str());
        cfg.dbname(settings.db.db.as_str());
        cfg.application_name(settings.db.application_name.as_str());
        cfg.connect_timeout(Duration::from_secs(
            settings.db.connect_timeout_seconds as u64,
        ));
        cfg
    }
}
