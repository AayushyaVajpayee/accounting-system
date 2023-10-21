use std::io;
use actix_web::{App, HttpServer};
use actix_web::middleware::Logger;

use crate::tenant::tenant_http_api;

mod ledger;
mod accounting;
mod seeddata;

mod configurations;
mod audit_table;
mod invoicing;
mod masters;
mod tenant;
mod common_utils;

pub fn build_dependencies(){
    //1. seeddata dependencies
    // seed service
    //2.global dependencies which are likely to be used everywhere
    //audit table service
    //tenant service
    //user service
    //3.masters dependencies
    //pincode master service
    //city master service
    //state master service
    //country master service
    //address master service
    //currency master service
    //ledger master service
    //account type master service
    //4. functional dependencies
    // accounts service
    // ledger service
    // invoice template service
    // invoice no series service
}
#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    println!("{}", std::process::id());
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(|a|seeddata::seeddata_http_api::init_routes(a))
            .configure(accounting::currency::currency_http_api::init_routes)
            .configure(accounting::account::account_http_api::init_routes)
            .configure(tenant_http_api::init_routes)
            .configure(accounting::user::user_http_api::init_routes)

    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await.expect("TODO: panic message");
    Ok(())
}


