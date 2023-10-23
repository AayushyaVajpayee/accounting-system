use clap::{Parser, Subcommand, ValueEnum};
use postgres::{Client, NoTls};

#[derive(Parser)]
struct Cli{
    ///postgres host
    #[arg(short,long,default_value="localhost")]
    host:String,
    ///postgres port exposed for connection
    #[arg(short,long,default_value_t=5432)]
    port:u16,
    ///password of the postgres user
    #[arg(long,default_value="postgres")]
    pwd:String,
    ///user with which to authenticate in postgres
    #[arg(short,long,default_value="postgres")]
    user:String,
    #[arg(short,long)]
    schema_path:String,
    #[command(subcommand)]
    command:Option<MySubCommand>
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum,Subcommand)]
enum MySubCommand{
    CreateDatabase,
    CreateSchema

}
fn main() {
    let cli = Cli::parse();
    println!("Hello, world!");
}


fn execute_create_database_command(host:String,user:String,password:String,port:u16,db:String){
    todo!("this function will create postgres client connection, create database and create schema")
    //todo how to know if schema or db is already created? most likely maintain a table with
    //schema version column
    // let client = connect_to_postgres()
}

fn connect_to_postgres(host:String,user:String,password:String,port:u16,db:String)->Client{
    let conn_str=format!("host={host} user={user} password={password} port={port} dbname={db}");
    let mut client = Client::connect(&conn_str, NoTls).unwrap();
    client
}

fn create_database(client:&mut Client,path:&str){
    let path_= format!("{path}/postgres/database.sql");
    let fi = std::fs::read_to_string(path).unwrap();
    client.simple_query(&fi).unwrap();
}
fn create_schema(client:&mut Client,path:&str){
    let path = format!("{path}/postgres/schema.sql");
    let fi= std::fs::read_to_string(path).unwrap();
    client.simple_query(&fi).unwrap();
}
fn get_dbname_for_creation(path:&str)->String{
    let path = format!("{path}/postgres/database.sql");
    let fi = std::fs::read_to_string(path).unwrap();
    fi.split_whitespace()
        .filter(|e| !e.trim().is_empty() )
        .skip(2)
        .map(|e|e.replace(";",""))
        .next().unwrap()
}
