use std::fmt::format;
use std::sync::OnceLock;
use clap::{Parser, Subcommand, ValueEnum};
use deadpool_postgres::{ManagerConfig, Pool, RecyclingMethod, Runtime};
use postgres::{Client, NoTls};
use std::time::Duration;
use env_logger::init;
use tokio_postgres::Config;
use db_prep_tool::{ get_seed_service};

static CONNECTION_POOL: OnceLock<Pool> = OnceLock::new();

pub fn get_postgres_conn_pool() -> &'static Pool {
    let p= CONNECTION_POOL.get();
    p.unwrap()
}
pub fn init_pool(pool: Pool){
    CONNECTION_POOL.set(pool).expect("TODO: panic message");
}

#[derive(Parser)]
struct Cli {
    ///postgres host
    #[arg(short, long, default_value = "localhost")]
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

    #[command(subcommand)]
    command: Option<MySubCommand>,
}
#[derive( Clone, PartialEq, Eq, PartialOrd, Ord, Subcommand)]
enum MySubCommand {
    CreateDbAndSchema{
        #[arg(short, long, default_value = "accounting_system")]
        dbname:String,
        #[arg(short, long,default_value="d")]
        schema_path: String,

    },
    CreateSeedData{
        #[arg(short, long)]
        dbname:String
    },
}
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let pool=connect_to_postgres(&cli.host, &cli.user, &cli.pwd, cli.port, "postgres");

    match cli.command.unwrap(){
        MySubCommand::CreateDbAndSchema { dbname, .. } => {
            println!("creating database");
            let db_query = format!("create database {};",dbname);
            pool.get().await.unwrap().simple_query(db_query.as_str()).await.unwrap();
            println!("created database");
            let pool=connect_to_postgres(&cli.host, &cli.user, &cli.pwd, cli.port, dbname.as_str());
            init_pool(pool);
            let seed_ser = get_seed_service(CONNECTION_POOL.get().unwrap());
            seed_ser.copy_tables().await;

        }
        MySubCommand::CreateSeedData {..}=> {



        }
    }

    println!("Hello, world!");
}



pub fn connect_to_postgres(
    host: &str,
    user: &str,
    password: &str,
    port: u16,
    db: &str,
)->Pool {
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