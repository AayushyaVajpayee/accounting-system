use std::sync::Arc;
use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use uuid::Uuid;
use web::{Data, Path};

use crate::accounting::currency::currency_models::CreateCurrencyMasterRequest;
use crate::accounting::currency::currency_service::{CurrencyService, CurrencyServiceError, get_currency_service};
use crate::common_utils::dao_error::DaoError;

impl ResponseError for CurrencyServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            CurrencyServiceError::Db(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            CurrencyServiceError::Db(e) => {
                HttpResponse::build(self.status_code()).json(e.to_string())
            }
        }
    }
}
async fn get_currency_by_id(
    id: Path<Uuid>,
    data: Data<Arc<dyn CurrencyService>>)
    -> actix_web::Result<impl Responder> {
    let p = data.get_currency_entry(&id).await?;
    Ok(web::Json(p))
}

async fn create_currency(
    request: web::Json<CreateCurrencyMasterRequest>,
    data: Data<Arc<dyn CurrencyService>>,
) -> actix_web::Result<impl Responder> {
    let p = data.create_currency_entry(&request.0).await?;
    Ok(web::Json(p))
}

pub fn init_routes(config: &mut web::ServiceConfig, currency_service: Arc<dyn CurrencyService>) {
    let data = Data::new(currency_service);
    config.service(
        map_endpoints_to_functions()
            .app_data(data));
}

fn map_endpoints_to_functions() -> Scope {
    web::scope("/currency")
        .route("/id/{id}", web::get().to(get_currency_by_id))
        .route("/create", web::post().to(create_currency))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use actix_web::{App, test};
    use async_trait::async_trait;
    use uuid::Uuid;

    use crate::accounting::currency::currency_http_api::map_endpoints_to_functions;
    use crate::accounting::currency::currency_models::{a_currency_master, CreateCurrencyMasterRequest, CurrencyMaster};
    use crate::accounting::currency::currency_service::{CurrencyService, MockCurrencyService};


    #[tokio::test]
    async fn test_api() {
        let mut currency_mock = MockCurrencyService::new();
        currency_mock.expect_create_currency_entry().returning(|_| Ok(Default::default()));
        let p = a_currency_master(Default::default());
        currency_mock.expect_get_currency_entry().returning(move |_| Ok(Some(p.clone())));
        let mock: Arc<dyn CurrencyService> = Arc::new(currency_mock);
        let id = Uuid::default();
        let currency_expected = mock.get_currency_entry(&id).await.unwrap().unwrap();
        let app_data = actix_web::web::Data::new(mock);
        let app = App::new()
            .service(map_endpoints_to_functions())
            .app_data(app_data);
        let app_service = test::init_service(app).await;
        let uri = format!("/currency/id/{}", Uuid::default());
        let request = test::TestRequest::get()
            .uri(uri.as_str())
            .to_request();
        let res: CurrencyMaster = test::call_and_read_body_json(&app_service, request).await;
        assert_eq!(res, currency_expected);
        let request = test::TestRequest::post()
            .uri("/currency/create")
            .set_json(CreateCurrencyMasterRequest { ..Default::default() })
            .to_request();
        let _: Uuid = test::call_and_read_body_json(&app_service, request).await;
    }
}