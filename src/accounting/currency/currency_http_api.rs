use actix_web::{Responder, web};
use deadpool_postgres::Pool;
use web::{Data, Path};
use crate::accounting::currency::currency_models::CreateCurrencyMasterRequest;
use crate::accounting::currency::currency_service::{CurrencyService, get_currency_service};
use crate::accounting::postgres_factory::get_postgres_conn_pool;

async fn get_currency_by_id(
    id: Path<i16>,
    data: Data<Box<dyn CurrencyService + Send + Sync>>)
    -> actix_web::Result<impl Responder> {
    let p = data.get_currency_entry(&id).await;
    Ok(web::Json(p))
}

async fn create_currency(
    request: web::Json<CreateCurrencyMasterRequest>,
    data: Data<Box<dyn CurrencyService + Send + Sync>>,
) -> actix_web::Result<impl Responder> {
    let p = data.create_currency_entry(&request.0).await;
    Ok(web::Json(p))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    let conn_pool = get_currency_service();
    let data = Data::new(conn_pool);
    config.service(
        web::scope("/currency")
            .app_data(data)
            .route("/id/{id}",
                   web::get().to(get_currency_by_id))
    );
}

