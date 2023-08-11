use std::collections::HashSet;
use std::pin::Pin;
use async_std::io::WriteExt;
use bytes::Bytes;
use deadpool_postgres::Pool;
use futures_util::{SinkExt, stream};
use pin_utils::pin_mut;

use postgres::{Client, NoTls};
use tokio::pin;
use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::Error;
use tokio_postgres::types::Type;
use crate::accounting::postgres_factory::{get_postgres_conn_pool, get_postgres_conn_pool1};

use crate::seeddata::constants::{FUNCTIONS_AND_PROCEDURES_SCRIPT_PATH, SCHEMA_CREATION_SCRIPT_PATH, SEED_FILES, SEED_FILES_LOCATION};

pub fn get_seed_filenames_ordered() -> Vec<String> {
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

#[allow(dead_code)]
fn read_csv(_name: &str) -> String {
    // let k = BinaryCopyInWriter::new(/* postgres::CopyInWriter<'_> */, /* &[postgres::types::Type] */);
    todo!()
}

#[allow(dead_code)]
fn create_postgres_client(port: u16) -> Client {
    let con_str =
        format!("host=localhost user=postgres password=postgres dbname=postgres port={port}");
    let client = Client::
    connect(&con_str, NoTls)
        .unwrap();
    client
}

async fn create_schema(client: &'static Pool) {
    let fi = std::fs::read_to_string(SCHEMA_CREATION_SCRIPT_PATH).unwrap();
    client.get().await
        .unwrap()
        .simple_query(&fi)
        .await
        .unwrap();
}

async fn create_functions_and_procedures(client: &'static Pool) {
    let fi = std::fs::read_to_string(FUNCTIONS_AND_PROCEDURES_SCRIPT_PATH).unwrap();
    client.get().await.unwrap().simple_query(&fi).await.unwrap();
}

pub async fn copy_tables(port: u16) {
    let file_names = get_seed_filenames_ordered();
    validate_seed_file_names(&file_names).unwrap();
    let client = get_postgres_conn_pool1(port);
    create_schema(client).await;
    create_functions_and_procedures(client).await;
    let mut conn = client.get().await.unwrap();
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

        let ss = stream::once(content);
        let copy_in_writer = client.get().await.unwrap().copy_in(&query).await.unwrap();
        pin_mut!(copy_in_writer);
        pin!(ss);
        copy_in_writer.send_all(&mut ss).await.unwrap();
        copy_in_writer.finish().await.unwrap();
        // let writer = BinaryCopyInWriter::new(copy_in_writer, &stm[..]);
        // pin!(writer);
        // copy_in_writer.write_all(content.as_ref()).unwrap();
        // let _rows_written = copy_in_writer.finish().await.unwrap();
    }

    txn.commit().await.unwrap();
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

    // #[test]
    // fn test_2() {
    //     let port =get_postgres_image_port().await;
    //     copy_tables(port)
    // }
}