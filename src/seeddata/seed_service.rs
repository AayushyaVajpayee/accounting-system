use std::collections::HashSet;

use async_std::io::WriteExt;
use async_trait::async_trait;
use bytes::Bytes;
use deadpool_postgres::Pool;
use futures_util::{SinkExt, stream};
use pin_utils::pin_mut;
use tokio::pin;
use tokio_postgres::Error;
use crate::accounting::postgres_factory::get_postgres_conn_pool;

use crate::seeddata::constants::{FUNCTIONS_AND_PROCEDURES_SCRIPT_PATH, SCHEMA_CREATION_SCRIPT_PATH, SEED_FILES, SEED_FILES_LOCATION};

#[async_trait]
pub trait SeedService {
    async fn copy_tables(&self);
}

struct SeedServiceImpl {
    pool: &'static Pool,
}

#[async_trait]
impl SeedService for SeedServiceImpl {
    async fn copy_tables(&self) {
        let pool = self.pool;
        let file_names = get_seed_filenames_ordered();
        validate_seed_file_names(&file_names).unwrap();
        create_schema(pool).await;
        create_functions_and_procedures(pool).await;
        let mut conn = pool.get().await.unwrap();
        let txn = conn.transaction().await.unwrap();
        let table_names = file_names
            .iter()
            .map(|f| f.split('.')
                .next()
                .unwrap().to_string()
            ).collect::<Vec<String>>();
        for t in table_names {
            let file_path = format!("{SEED_FILES_LOCATION}{t}.csv");
            let content =
                async {
                    Ok::<_, Error>(Bytes::from(
                        tokio::fs::read_to_string(file_path).await.unwrap()
                    ))
                };
            let query = format!("copy {t} from stdin with csv header");

            let stream = stream::once(content);
            let copy_in_writer = txn.copy_in(&query).await.unwrap();
            pin_mut!(copy_in_writer);
            pin!(stream);
            copy_in_writer.send_all(&mut stream).await.unwrap();
            copy_in_writer.finish().await.unwrap();
        }
        txn.commit().await.unwrap();
    }
}


pub fn get_seed_service() -> Box<dyn SeedService + Send + Sync> {
    let pool = get_postgres_conn_pool();
    let seed_s = SeedServiceImpl { pool };
    Box::new(seed_s)
}

pub fn get_seed_service_with_pool_supplied(pool: &'static Pool) -> Box<dyn SeedService + Send + Sync> {
    let seed_s = SeedServiceImpl { pool };
    Box::new(seed_s)
}
fn get_seed_filenames_ordered() -> Vec<String> {
    let path = format!("{}{}", SEED_FILES_LOCATION, SEED_FILES);
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|row| !row.is_empty())
        .skip(1)
        .map(|row| row.to_string())
        .collect::<Vec<String>>()
}

fn validate_seed_file_names(names: &Vec<String>) -> Result<(), Vec<String>> {
    let available_files = std::fs::read_dir(SEED_FILES_LOCATION).unwrap()
        .map(|row| row.unwrap().file_name().to_str().unwrap().to_string())
        .collect::<HashSet<String>>();
    let p = names.iter().map(|r|
        if !available_files.contains(r) {
            r.clone()
        } else {
            "".to_string()
        }
    ).filter(|r| !r.is_empty()).collect::<Vec<String>>();
    if !p.is_empty() {
        Err(p)
    } else {
        Ok(())
    }
}


async fn create_schema(client: &Pool) {
    let fi = std::fs::read_to_string(SCHEMA_CREATION_SCRIPT_PATH).unwrap();
    client.get().await
        .unwrap()
        .simple_query(&fi)
        .await
        .unwrap();
}

async fn create_functions_and_procedures(client: &Pool) {
    let fi = std::fs::read_to_string(FUNCTIONS_AND_PROCEDURES_SCRIPT_PATH).unwrap();
    client.get().await.unwrap().simple_query(&fi).await.unwrap();
}


#[cfg(test)]
mod tests {
    use crate::seeddata::seed_service::{get_seed_filenames_ordered, validate_seed_file_names};

    #[test]
    fn test_k() {
        let k = get_seed_filenames_ordered();
        println!("{:?}", k);
        let kk = validate_seed_file_names(&k);
        println!("{:?}", kk);
    }
}