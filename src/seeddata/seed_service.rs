use std::collections::HashSet;
use std::io::Write;

use postgres::{Client, NoTls};

use crate::seeddata::constants::{SCHEMA_CREATION_SCRIPT_PATH, SEED_FILES, SEED_FILES_LOCATION};

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

fn read_csv(name: &str) -> String {
    // let k = BinaryCopyInWriter::new(/* postgres::CopyInWriter<'_> */, /* &[postgres::types::Type] */);
    todo!()
}
fn create_postgres_client(port: u16) -> Client {
    let con_str =
        format!("host=localhost user=postgres password=postgres dbname=postgres port={port}");
    let mut client = Client::
    connect(&con_str, NoTls)
        .unwrap();
    client
}
fn create_schema(client: &mut Client) {
    let fi = std::fs::read_to_string(SCHEMA_CREATION_SCRIPT_PATH).unwrap();
    client.simple_query(&fi).unwrap();
}
pub fn copy_tables(port: u16) {
    let filenames = get_seed_filenames_ordered();
    validate_seed_file_names(&filenames).unwrap();
    let mut client = create_postgres_client(port);
    create_schema(&mut client);
    let mut txn = client.transaction().unwrap();
    let tablenames = filenames
        .iter()
        .map(|f| f.split(".")
            .next()
            .unwrap().to_string()
        ).collect::<Vec<String>>();
    tablenames.iter().for_each(|t| {
        let file_path = format!("{SEED_FILES_LOCATION}{t}.csv");
        let content = std::fs::read_to_string(file_path).unwrap();
        let query = format!("copy {t} from stdin with csv header");
        let mut k = txn.copy_in(&query).unwrap();
        k.write_all(content.as_ref()).unwrap();
        let rows_written = k.finish().unwrap();
        println!("rows written {rows_written}");
    });
    txn.commit().unwrap();
}


#[cfg(test)]
mod tests {
    use crate::seeddata::seed_service::{copy_tables, get_seed_filenames_ordered, validate_seed_file_names};
    use crate::test_utils::test_utils_postgres::run_postgres;

    #[test]
    fn test_k() {
        let mut k = get_seed_filenames_ordered();
        println!("{:?}", k);
        let kk = validate_seed_file_names(&k);
        println!("{:?}", kk);
    }

    #[test]
    fn test_2() {
        let node = run_postgres();
        let port = node.get_host_port_ipv4(5432);
        copy_tables(port)
    }
}