use std::io;
use std::sync::Arc;

use actix_web::{App, HttpResponseBuilder, HttpServer, Responder, ResponseError, web};
use actix_web::body::MessageBody;
use actix_web::dev::Service;
use actix_web::http::StatusCode;
use actix_web::middleware::Logger;
use actix_web_lab::middleware::from_fn;

use crate::accounting::account::account_service::get_account_service;
use crate::accounting::account::account_type::account_type_service::get_account_type_master_service;
use crate::accounting::currency::currency_service::get_currency_service;
use crate::accounting::postgres_factory::get_postgres_conn_pool;
use crate::accounting::user::user_service::get_user_service;
use crate::audit_table::audit_service::get_audit_service;
use crate::common_utils::pagination::pagination_utils::pagination_header_middleware;
use crate::ledger::ledger_transfer_service::get_ledger_transfer_service;
use crate::ledger::ledgermaster::ledger_master_service::get_ledger_master_service;
use crate::masters::city_master::city_master_service::get_city_master_service;
use crate::masters::company_master::company_master_service::get_company_master_service;
use crate::masters::country_master::country_service::get_country_master_service;
use crate::masters::pincode_master::pincode_master_service::get_pincode_master_service;
use crate::masters::state_master::state_master_service::get_state_master_service;
use crate::tenant::tenant_http_api;
use crate::tenant::tenant_service::get_tenant_service;

mod ledger;
mod accounting;


mod configurations;
mod audit_table;
mod invoicing;
mod masters;
mod tenant;
mod common_utils;
mod db_schema_syncer;


pub fn build_dependencies() {
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
    // ledger transfer service
    // invoice template service
    // invoice no series service
}

async fn healthcheck() -> actix_web::Result<impl Responder> {
    let builder = HttpResponseBuilder::new(StatusCode::OK);
    builder.await
}




#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let pool = Arc::new(get_postgres_conn_pool());
    let audit_table_service = get_audit_service(pool.clone());
    let tenant_service = get_tenant_service(pool.clone());
    let user_service = get_user_service(pool.clone());
    let pincode_service = get_pincode_master_service(pool.clone());
    let city_master_service = get_city_master_service(pool.clone());
    let state_master_service = get_state_master_service(pool.clone());
    let country_master_service = get_country_master_service(pool.clone());
    // let address_master_service = get_address_master_service(pool.clone());
    let currency_service = get_currency_service(pool.clone());
    let ledger_master_service = get_ledger_master_service(pool.clone());
    let account_type_master_service = get_account_type_master_service(pool.clone());
    let account_service = get_account_service(pool.clone());
    let ledger_service = get_ledger_transfer_service(pool.clone());
    let company_master_service = get_company_master_service(pool.clone(), tenant_service.clone(), user_service.clone());
    // let invoice_template_service= get_invoice_template_service();
    println!("{}", std::process::id());
    HttpServer::new(move || {
        App::new()
            .wrap(from_fn(pagination_header_middleware))
            .wrap(Logger::default())
            .configure(|conf| audit_table::audit_table_http_api::init_routes(conf, audit_table_service.clone()))
            .configure(|conf| tenant_http_api::init_routes(conf, tenant_service.clone()))
            .configure(|conf| accounting::user::user_http_api::init_routes(conf, user_service.clone()))
            .configure(|conf| masters::pincode_master::pincode_http_api::init_routes(conf, pincode_service.clone()))
            .configure(|conf| masters::city_master::city_master_http_api::init_routes(conf, city_master_service.clone()))
            .configure(|conf| masters::state_master::state_master_http_api::init_routes(conf, state_master_service.clone()))
            .configure(|conf| masters::country_master::country_master_http_api::init_routes(conf, country_master_service.clone()))
            .configure(|conf| masters::company_master::company_master_http_api::init_routes(conf, company_master_service.clone()))
            .configure(|conf| accounting::currency::currency_http_api::init_routes(conf, currency_service.clone()))
            .configure(|conf| ledger::ledgermaster::ledger_master_http_api::init_routes(conf, ledger_master_service.clone()))
            .configure(|conf| accounting::account::account_type::account_type_http_api::init_routes(conf, account_type_master_service.clone()))
            .configure(|conf| accounting::account::account_http_api::init_routes(conf, account_service.clone()))
            .configure(|conf| ledger::ledger_transfer_http_api::init_routes(conf, ledger_service.clone()))
            .route("/healthcheck", web::get().to(healthcheck))
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await.expect("TODO: panic message");
    Ok(())
}


