use std::io;
use actix_web::{App, HttpServer};
use actix_web::middleware::Logger;
use crate::accounting::tenant::tenant_http_api_routes::init_routes;

mod ledger;
mod accounting;
mod seeddata;

mod configurations;

#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    println!("{}", std::process::id());
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(init_routes)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await.expect("TODO: panic message");
    Ok(())
}


