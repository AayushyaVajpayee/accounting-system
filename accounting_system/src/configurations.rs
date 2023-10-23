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

pub fn get_dev_conf() -> Setting {
    config::Config::builder()
        .add_source(config::File::with_name("conf/conf.toml"))
        .build()
        .unwrap().try_deserialize().unwrap()
}


#[cfg(test)]
pub mod configuration_test_code {
    use serde::Deserialize;

    use crate::configurations::Database;

    #[derive(Debug, Deserialize)]
    pub struct DockerPostgresDetail {
        pub image: String,
        pub image_tag: String,
    }

    #[derive(Debug, Deserialize)]
    #[allow(unused)]
    pub struct TestSettings {
        pub db: Database,
        pub docker_postgres_detail: DockerPostgresDetail,
    }

    pub fn get_tests_conf() -> TestSettings {
        config::Config::builder()
            .add_source(config::File::with_name("conf/tests_conf.toml"))
            .build()
            .unwrap().try_deserialize().unwrap()
    }
}