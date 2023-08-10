use std::io;
use actix_web::{App, HttpServer};
use crate::accounting::tenant::tenant_http_api_routes::init_routes;

mod ledger;
mod accounting;
mod seeddata;
mod test_utils;

#[actix_web::main]
async fn main() -> io::Result<()> {
    println!("{}", std::process::id());
    HttpServer::new(move || {
        App::new()
            .configure(init_routes)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await.expect("TODO: panic message");
    Ok(())
}


