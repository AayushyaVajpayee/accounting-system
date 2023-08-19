use actix_web::{Responder, web};
use deadpool_postgres::Pool;
use crate::accounting::postgres_factory::get_postgres_conn_pool;
use crate::seeddata::seed_service::copy_tables;


pub async fn init_schema_and_create_seed_data(data: web::Data<Pool>) -> actix_web::Result<impl Responder> {
    copy_tables(&data).await;
    Ok("oh yeah seed data".to_string())
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    let conn_pool = get_postgres_conn_pool().clone();
    let data = web::Data::new(conn_pool);
    config.service(web::scope("/seeddata")
        .app_data(data)
        .route("/initdb", web::get().to(init_schema_and_create_seed_data)))
    ;
}