use std::collections::HashSet;
use std::io::Write;
use std::ops::Not;

use postgres::{Client, NoTls};

const SEED_FILES: &'static str = "seed_files_metadata.csv";


pub fn get_seed_filenames_ordered() -> Vec<String> {
    let path = format!("schema/postgres/seed_data/seed_files_metadata.csv");
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|row| !row.is_empty())
        .skip(1)
        .map(|row| row.to_string())
        .collect::<Vec<String>>()
}

fn validate_seed_file_names(names: &Vec<String>) -> Result<(), Vec<String>> {
    let available_files = std::fs::read_dir("schema/postgres/seed_data/").unwrap()
        .map(|row| row.unwrap().file_name().to_str().unwrap().to_string())
        .collect::<HashSet<String>>();
    let p = names.iter().map(|r|
        if (!available_files.contains(r)) {
            r.clone()
        } else {
            "".to_string()
        }
    ).filter(|r| !r.is_empty()).collect::<Vec<String>>();
    if (!p.is_empty()) {
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
    let path = format!("schema/postgres/schema.sql");
    let fi = std::fs::read_to_string(path).unwrap();
    client.simple_query(&fi).unwrap();
}

fn copy_tables(port: u16) {
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
        let file_path = format!("schema/postgres/seed_data/{t}.csv");
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
    use testcontainers::clients;
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;

    use crate::seeddata::seed_service::{copy_tables, get_seed_filenames_ordered, validate_seed_file_names};

    #[test]
    fn test_k() {
        let mut k = get_seed_filenames_ordered();
        println!("{:?}", k);
        let kk = validate_seed_file_names(&k);
        println!("{:?}", kk);
    }

    #[test]
    fn test_2() {
        let test_container_client = clients::Cli::default();
        let image = "postgres";
        let image_tag = "latest";
        let generic_postgres = GenericImage::new(image, image_tag)
            .with_wait_for(WaitFor::message_on_stderr("database system is ready to accept connections"))
            .with_env_var("POSTGRES_DB", "postgres")
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres");
        let node = test_container_client.run(generic_postgres);
        let port = node.get_host_port_ipv4(5432);
        copy_tables(port)
    }
}