use std::io;
use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponseBuilder, HttpServer, Responder};
use actix_web_lab::middleware::from_fn;
use log::LevelFilter;

use crate::accounting::account::account_service::get_account_service;
use crate::accounting::account::account_type::account_type_service::get_account_type_master_service;
use crate::accounting::currency::currency_service::get_currency_service;
use crate::accounting::postgres_factory::get_postgres_conn_pool;
use crate::accounting::user::user_service::get_user_service;
use crate::audit_table::audit_service::get_audit_service;
use crate::common_utils::pagination::pagination_utils::pagination_header_middleware;
use crate::common_utils::utils::tenant_user_header_middleware;
use crate::invoicing::invoice_template::invoice_template_service::get_invoice_template_master_service;
use crate::invoicing::invoicing_series::invoicing_series_service::get_invoicing_series_service;
use crate::invoicing::invoicing_service::get_invoicing_service;
use crate::ledger::ledger_transfer_service::get_ledger_transfer_service;
use crate::ledger::ledgermaster::ledger_master_service::get_ledger_master_service;
use crate::masters::address_master::address_service::get_address_service;
use crate::masters::business_entity_master::business_entity_service::get_business_entity_master_service;
use crate::masters::city_master::city_master_service::get_city_master_service;
use crate::masters::company_master::company_master_service::get_company_master_service;
use crate::masters::country_master::country_service::get_country_master_service;
use crate::masters::pincode_master::pincode_master_service::get_pincode_master_service;
use crate::masters::product_item_master::product_item_service::get_product_item_service;
use crate::masters::state_master::state_master_service::get_state_master_service;
use crate::storage::storage_service::get_storage_service;
use crate::tenant::tenant_http_api;
use crate::tenant::tenant_service::get_tenant_service;

mod accounting;
mod ledger;

mod audit_table;
mod common_utils;
mod configurations;
mod db_schema_syncer;
mod invoicing;
mod masters;
mod storage;
mod tenant;

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
    std::env::set_var("RUST_LOG", "error");
    env_logger::builder()
        .filter(Some("actix"), LevelFilter::Info)
        .filter(Some("actix_web"), LevelFilter::Info)
        .filter(Some("accounting_system"), LevelFilter::Info)
        .format_module_path(true)
        .format_timestamp_micros()
        .init();
    let storage = get_storage_service().await;
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
    let company_master_service =
        get_company_master_service(pool.clone(), tenant_service.clone(), user_service.clone());
    let address_service = get_address_service(
        pool.clone(),
        country_master_service.clone(),
        city_master_service.clone(),
        pincode_service.clone(),
        state_master_service.clone(),
    );
    let business_entity_service =
        get_business_entity_master_service(pool.clone(), address_service.clone());
    let invoice_template_service = get_invoice_template_master_service(pool.clone());
    let invoicing_series_service = get_invoicing_series_service(pool.clone());
    let product_item_serv = get_product_item_service(pool.clone());
    let invoicing_service = get_invoicing_service(
        pool.clone(),
        tenant_service.clone(),
        currency_service.clone(),
        invoicing_series_service.clone(),
        business_entity_service.clone(),
        invoice_template_service.clone(),
        storage.clone(),
        product_item_serv.clone(),
    );
    // let invoice_template_service= get_invoice_template_service();
    println!("{}", std::process::id());
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(tenant_service.clone())
            .app_data(user_service.clone())
            .wrap(from_fn(tenant_user_header_middleware))
            .wrap(from_fn(pagination_header_middleware))
            .configure(|conf| {
                audit_table::audit_table_http_api::init_routes(conf, audit_table_service.clone())
            })
            .configure(|conf| tenant_http_api::init_routes(conf, tenant_service.clone()))
            .configure(|conf| {
                accounting::user::user_http_api::init_routes(conf, user_service.clone())
            })
            .configure(|conf| {
                masters::pincode_master::pincode_http_api::init_routes(
                    conf,
                    pincode_service.clone(),
                )
            })
            .configure(|conf| {
                masters::city_master::city_master_http_api::init_routes(
                    conf,
                    city_master_service.clone(),
                )
            })
            .configure(|conf| {
                masters::state_master::state_master_http_api::init_routes(
                    conf,
                    state_master_service.clone(),
                )
            })
            .configure(|conf| {
                masters::country_master::country_master_http_api::init_routes(
                    conf,
                    country_master_service.clone(),
                )
            })
            .configure(|conf| {
                masters::company_master::company_master_http_api::init_routes(
                    conf,
                    company_master_service.clone(),
                )
            })
            .configure(|conf| {
                accounting::currency::currency_http_api::init_routes(conf, currency_service.clone())
            })
            .configure(|conf| {
                ledger::ledgermaster::ledger_master_http_api::init_routes(
                    conf,
                    ledger_master_service.clone(),
                )
            })
            .configure(|conf| {
                accounting::account::account_type::account_type_http_api::init_routes(
                    conf,
                    account_type_master_service.clone(),
                )
            })
            .configure(|conf| {
                accounting::account::account_http_api::init_routes(conf, account_service.clone())
            })
            .configure(|conf| {
                ledger::ledger_transfer_http_api::init_routes(conf, ledger_service.clone())
            })
            .configure(|conf| {
                invoicing::invoicing_http_api::init_routes(conf, invoicing_service.clone())
            })
            .configure(|conf| {
                masters::product_item_master::product_item_http_api::init_routes(
                    conf,
                    product_item_serv.clone(),
                )
            })
            .configure(|conf| {
                invoicing::invoicing_series::invoicing_series_http_api::init_routes(
                    conf,
                    invoicing_series_service.clone(),
                )
            })
            .configure(|conf|{
                masters::address_master::address_http_api::init_routes(conf,address_service.clone())
            })
            .configure(|conf|{
                masters::business_entity_master::business_entity_http_api::init_routes(conf,business_entity_service.clone())
            })
            .route("/healthcheck", web::get().to(healthcheck))
    })
    .bind(("0.0.0.0", 8090))?
    .run()
    .await
    .expect("TODO: panic message");
    Ok(())
}
