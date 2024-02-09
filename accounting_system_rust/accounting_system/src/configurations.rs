use config::{Environment, Source};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Database {
    pub port: u16,
    pub user: String,
    pub db: String,
    pub max_connections: u16,
    pub host: String,
    pub connect_timeout_seconds: u16,
    pub password: String,
    pub wait_timeout_seconds: u16,
    pub recycling_method: String,
    pub application_name: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Setting {
    pub db: Database,
}
#[allow(dead_code)]
pub fn get_dev_conf() -> Setting {
    let p = Environment::with_prefix("POSTGRES").prefix_separator("_");
    let vars = p.collect().unwrap();
    vars.iter().for_each(|a| {
        println!("{} -------- \"{}\"", a.0, a.1);
    });
    Setting {
        db: Database {
            port: vars.get("port").unwrap().clone().into_uint().unwrap() as u16,
            user: vars.get("user").unwrap().clone().into_string().unwrap(),
            db: vars.get("app_db").unwrap().clone().into_string().unwrap(),
            max_connections: vars.get("max_connections").unwrap().clone().into_uint().unwrap() as u16,
            host: vars.get("host").unwrap().clone().into_string().unwrap(),
            connect_timeout_seconds: vars
                .get("connect_timeout_seconds")
                .unwrap().clone()
                .into_uint()
                .unwrap() as u16,
            password: vars.get("password").unwrap().clone().into_string().unwrap(),
            wait_timeout_seconds: vars
                .get("wait_timeout_seconds")
                .unwrap().clone()
                .into_uint()
                .unwrap() as u16,
            recycling_method: vars.get("pool_recycling_method").unwrap().clone().into_string().unwrap(),
            application_name: vars.get("application_name").unwrap().clone().into_string().unwrap(),
        },
    }
}