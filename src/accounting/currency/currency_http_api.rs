use std::sync::Arc;
use actix_web::{Responder, Scope, web};
use web::{Data, Path};

use crate::accounting::currency::currency_models::CreateCurrencyMasterRequest;
use crate::accounting::currency::currency_service::{CurrencyService, get_currency_service};

async fn get_currency_by_id(
    id: Path<i16>,
    data: Data<Arc<dyn CurrencyService>>)
    -> actix_web::Result<impl Responder> {
    let p = data.get_currency_entry(&id).await;
    Ok(web::Json(p))
}

async fn create_currency(
    request: web::Json<CreateCurrencyMasterRequest>,
    data: Data<Arc<dyn CurrencyService>>,
) -> actix_web::Result<impl Responder> {
    let p = data.create_currency_entry(&request.0).await;
    Ok(web::Json(p))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    let conn_pool = get_currency_service();
    let data = Data::new(conn_pool);
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

    use crate::accounting::currency::currency_http_api::map_endpoints_to_functions;
    use crate::accounting::currency::currency_models::{CreateCurrencyMasterRequest, CurrencyMaster};
    use crate::accounting::currency::currency_service::CurrencyService;

    struct MockCurrencyService {}

    #[async_trait]
    impl CurrencyService for MockCurrencyService {
        async fn create_currency_entry(&self, _request: &CreateCurrencyMasterRequest) -> i16 {
            0
        }

        async fn get_currency_entry(&self, _id: &i16) -> Option<CurrencyMaster> {
            Some(Default::default())
        }
    }

    #[tokio::test]
    async fn test_api() {
        let mock: Arc<dyn CurrencyService> = Arc::new(MockCurrencyService {});
        let tenant_expected = mock.get_currency_entry(&1).await.unwrap();
        let app_data = actix_web::web::Data::new(mock);
        let app = App::new()
            .service(map_endpoints_to_functions())
            .app_data(app_data);
        let app_service = test::init_service(app).await;
        let request = test::TestRequest::get()
            .uri("/currency/id/1")
            .to_request();
        let res: CurrencyMaster = test::call_and_read_body_json(&app_service, request).await;
        assert_eq!(res, tenant_expected);
        let request = test::TestRequest::post()
            .uri("/currency/create")
            .set_json(CreateCurrencyMasterRequest { ..Default::default() })
            .to_request();
        let _: i32 = test::call_and_read_body_json(&app_service, request).await;
    }
}