use std::sync::Arc;

use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use uuid::Uuid;
use web::{Data, Path};

use crate::accounting::currency::currency_models::CreateCurrencyMasterRequest;
use crate::accounting::currency::currency_service::{
    CurrencyService, CurrencyServiceError,
};

impl ResponseError for CurrencyServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            CurrencyServiceError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
    data: Data<Arc<dyn CurrencyService>>,
) -> actix_web::Result<impl Responder> {
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
    config.service(map_endpoints_to_functions().app_data(data));
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
    use uuid::Uuid;

    use crate::accounting::currency::currency_http_api::map_endpoints_to_functions;
    use crate::accounting::currency::currency_models::{
        a_currency_master, CreateCurrencyMasterRequest, CurrencyMaster,
    };
    use crate::accounting::currency::currency_service::{CurrencyService, MockCurrencyService};
    use crate::get_and_create_api_test;

    #[tokio::test]
    async fn test_api() {
        let p = a_currency_master(Default::default());
        let p1=p.clone();
        let closure = || {
            let mut currency_mock = MockCurrencyService::new();
            currency_mock
                .expect_create_currency_entry()
                .returning(|_| Ok(Default::default()));
            currency_mock
                .expect_get_currency_entry()
                .returning(move |_| Ok(Some(p1.clone())));
            currency_mock
        };
        let get_uri = format!("/currency/id/{}", Uuid::default());
        get_and_create_api_test!(CurrencyMaster,CurrencyService,closure,get_uri,
            "/currency/create",
            CreateCurrencyMasterRequest {
                ..Default::default()
            },
            p
        );
    }
}
