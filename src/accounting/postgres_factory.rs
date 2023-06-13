use postgres::{Client, NoTls};

#[cfg(test)]
pub fn create_postgres_client_for_test(port: u16) -> Client {
    let con_str =
        format!("host=localhost user=postgres password=postgres dbname=postgres port={port}");
    let client = Client::
    connect(&con_str, NoTls)
        .unwrap();
    client
}

pub fn create_postgres_client()->Client{
    let con_str =
        format!("host=localhost user=postgres password=postgres dbname=postgres port=5432");
    let client = Client::
    connect(&con_str, NoTls)
        .unwrap();
    client

}