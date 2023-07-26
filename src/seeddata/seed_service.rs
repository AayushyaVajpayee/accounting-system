use std::collections::HashSet;
use std::io::Write;

use postgres::{Client, NoTls};

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

fn create_schema(client: &mut Client) {
    let fi = std::fs::read_to_string(SCHEMA_CREATION_SCRIPT_PATH).unwrap();
    client.simple_query(&fi).unwrap();
}

fn create_functions_and_procedures(client: &mut Client) {
    let fi = std::fs::read_to_string(FUNCTIONS_AND_PROCEDURES_SCRIPT_PATH).unwrap();
    client.simple_query(&fi).unwrap();
}

pub fn copy_tables(port: u16) {
    let file_names = get_seed_filenames_ordered();
    validate_seed_file_names(&file_names).unwrap();
    let mut client = create_postgres_client(port);
    create_schema(&mut client);
    create_functions_and_procedures(&mut client);
    let mut txn = client.transaction().unwrap();
    let table_names = file_names
        .iter()
        .map(|f| f.split(".")
            .next()
            .unwrap().to_string()
        ).collect::<Vec<String>>();
    table_names.iter().for_each(|t| {
        let file_path = format!("{SEED_FILES_LOCATION}{t}.csv");
        let content = std::fs::read_to_string(file_path).unwrap();
        let query = format!("copy {t} from stdin with csv header");
        let mut copy_in_writer = txn.copy_in(&query).unwrap();
        copy_in_writer.write_all(content.as_ref()).unwrap();
        let _rows_written = copy_in_writer.finish().unwrap();
    });
    txn.commit().unwrap();
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
    //     let port =get_postgres_image_port();
    //     copy_tables(port)
    // }
}