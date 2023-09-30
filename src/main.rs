use std::io;
use actix_web::{App, HttpServer};
use actix_web::middleware::Logger;
mod ledger;
mod accounting;
mod seeddata;

mod configurations;
mod invoicing;
mod masters;

#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    println!("{}", std::process::id());
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(seeddata::seeddata_http_api::init_routes)
            .configure(accounting::currency::currency_http_api::init_routes)
            .configure(accounting::account::account_http_api::init_routes)
            .configure(accounting::tenant::tenant_http_api::init_routes)
            .configure(accounting::user::user_http_api::init_routes)

    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await.expect("TODO: panic message");
    Ok(())
}


