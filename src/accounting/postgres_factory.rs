use std::sync::OnceLock;
use std::time::Duration;
use deadpool_postgres::{ManagerConfig, Pool, Runtime};
use deadpool_postgres::RecyclingMethod::{Clean, Verified};
use tokio_postgres::{Config, NoTls};

static CONNECTION_POOL: OnceLock<Pool> = OnceLock::new();

pub fn get_postgres_conn_pool() -> &'static Pool {
    get_postgres_conn_pool1(5432)
}

pub fn get_postgres_conn_pool1(port: u16) -> &'static Pool {
    CONNECTION_POOL.get_or_init(|| {
        let mut cfg = Config::new();
        cfg.host("localhost");
        cfg.port(port);
        cfg.user("postgres");
        cfg.password("postgres");
        cfg.dbname("postgres");
        cfg.application_name("accounting-system");
        cfg.connect_timeout(Duration::from_secs(15));
        let mgr = deadpool_postgres::Manager::from_config(cfg, NoTls, ManagerConfig { recycling_method: Clean });
        Pool::builder(mgr)
            .max_size(5)
            .runtime(Runtime::Tokio1)
            .create_timeout(Some(Duration::from_secs(10)))
            .build().unwrap()
    })
}