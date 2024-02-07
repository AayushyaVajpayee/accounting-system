use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError, Scope, web};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use std::sync::Arc;
use uuid::Uuid;
use web::{Data, Path};

use crate::accounting::currency::currency_models::CreateCurrencyMasterRequest;
use crate::accounting::currency::currency_service::{
    CurrencyService, CurrencyServiceError,
};
use crate::common_utils::utils::extract_tenant_id_from_header;
use crate::setup_routes;

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
    request:HttpRequest
) -> actix_web::Result<impl Responder> {
    let tenant_id = extract_tenant_id_from_header(&request)?;
    let p = data.get_currency_entry(id.into_inner(),tenant_id).await?;
    Ok(web::Json(p))
}

async fn create_currency(
    request: web::Json<CreateCurrencyMasterRequest>,
    data: Data<Arc<dyn CurrencyService>>,
    req:HttpRequest
) -> actix_web::Result<impl Responder> {
    let tenant_id = extract_tenant_id_from_header(&req)?;
    let p = data.create_currency_entry(&request.0,tenant_id).await?;
    Ok(web::Json(p))
}
setup_routes!(CurrencyService,"/currency",
    "/id/{id}", web::get().to(get_currency_by_id),
    "/create",web::post().to(create_currency));

#[cfg(test)]
mod tests {
    use actix_web::{App, test};
    use std::sync::Arc;
    use uuid::Uuid;

    use crate::accounting::currency::currency_http_api::map_endpoints_to_functions;
    use crate::accounting::currency::currency_models::{
         CreateCurrencyMasterRequest, CurrencyMaster,
    };
    use crate::accounting::currency::currency_models::tests::a_currency_master;
    use crate::accounting::currency::currency_service::{CurrencyService, MockCurrencyService};
    use crate::{ get_and_create_api_test_v2};
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    #[tokio::test]
    async fn test_api() {
        let p = a_currency_master(Default::default());
        let p1=p.clone();
        let closure = || {
            let mut currency_mock = MockCurrencyService::new();
            currency_mock
                .expect_create_currency_entry()
                .returning(|_,_| Ok(Default::default()));
            currency_mock
                .expect_get_currency_entry()
                .returning(move |_,_| Ok(Some(p1.clone())));
            currency_mock
        };
        let get_uri = format!("/currency/id/{}", Uuid::default());
        get_and_create_api_test_v2!(CurrencyMaster,CurrencyService,closure,get_uri,
            "/currency/create",
            CreateCurrencyMasterRequest {
                ..Default::default()
            },
            p,
            *SEED_TENANT_ID
        );
    }
}
