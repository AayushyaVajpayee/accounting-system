use accounting_system::init_db_with_seed;
use clap::{Parser, Subcommand};
use deadpool_postgres::{ManagerConfig, Pool, RecyclingMethod, Runtime};
use postgres::NoTls;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio_postgres::Config;

#[derive(Parser)]
struct Cli {
    ///postgres host
    #[arg( long, default_value = "localhost")]
    host: String,
    ///postgres port exposed for connection
    #[arg(short, long, default_value_t = 5432)]
    port: u16,
    ///password of the postgres user
    #[arg(long, default_value = "postgres")]
    pwd: String,
    ///user with which to authenticate in postgres
    #[arg(short, long, default_value = "postgres")]
    user: String,
    #[arg(short, long, default_value = "postgres")]
    dbname: String,
    #[command(subcommand)]
    command: Option<MySubCommand>,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Subcommand)]
enum MySubCommand {
    CreateDbAndSchema {
        #[arg(short, long, default_value = "accounting_system")]
        dbname: String,
        #[arg(short, long, default_value = "d")]
        schema_path: String,
    },
    CreateSeedData {
        #[arg(short, long)]
        dbname: String,
    },

    DropAllDbs,
}
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let pool = connect_to_postgres(&cli.host, &cli.user, &cli.pwd, cli.port, &cli.dbname);

    match cli.command.unwrap() {
        MySubCommand::CreateDbAndSchema { dbname, .. } => {
            println!("creating database");
            let db_query = format!("create database {};", dbname);
            pool.get()
                .await
                .unwrap()
                .simple_query(db_query.as_str())
                .await
                .unwrap();
            println!("created database");
            let pool = Arc::new(connect_to_postgres(&cli.host, &cli.user, &cli.pwd, cli.port, dbname.as_str()));
            init_db_with_seed(pool.clone()).await;
        }
        MySubCommand::CreateSeedData { dbname } => {
            let pool = Arc::new(connect_to_postgres(&cli.host, &cli.user, &cli.pwd, cli.port, dbname.as_str()));
            init_db_with_seed(pool.clone()).await;
        }
        MySubCommand::DropAllDbs => {
            let k = pool.get().await.unwrap()
                .query("select datname from pg_database where datistemplate=false and datname!='postgres';",&[])
                .await.unwrap().iter().map(|a| a.get(0)).collect::<Vec<String>>();
            let pp = k
                .iter()
                .map(|a| format!("drop database {} with (FORCE);", a))
                .collect::<Vec<String>>();
            for x in pp {
                pool.get().await.unwrap().query(&x, &[]).await.unwrap();
            }
        }
    }

    println!("Hello, world!");
}

pub fn connect_to_postgres(host: &str, user: &str, password: &str, port: u16, db: &str) -> Pool {
    let mut cfg = Config::new();
    cfg.host(host);
    cfg.user(user);
    cfg.password(password);
    cfg.port(port);
    cfg.dbname(db);
    let mgr = deadpool_postgres::Manager::from_config(
        cfg,
        NoTls,
        ManagerConfig {
            recycling_method: RecyclingMethod::Clean,
        },
    );
    Pool::builder(mgr)
        .max_size(2)
        .runtime(Runtime::Tokio1)
        .create_timeout(Some(Duration::from_secs(5)))
        .build()
        .unwrap()
}
