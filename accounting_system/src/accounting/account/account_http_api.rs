use std::sync::Arc;

use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use uuid::Uuid;

use crate::accounting::account::account_models::CreateAccountRequest;
use crate::accounting::account::account_service::{AccountService, AccountServiceError};

impl ResponseError for AccountServiceError {
    fn status_code(&self) -> StatusCode {
        match self { AccountServiceError::Db(_) => { StatusCode::INTERNAL_SERVER_ERROR } }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            AccountServiceError::Db(e) => {
                HttpResponse::build(self.status_code()).json(e.to_string())
            }
        }
    }
}
async fn get_account_by_id(id: Path<Uuid>,
                           data: Data<Arc<dyn AccountService>>)
                           -> actix_web::Result<impl Responder> {
    let account = data.get_account_by_id(&id).await?;
    Ok(web::Json(account))
}


async fn create_account(request: web::Json<CreateAccountRequest>,
                        data: Data<Arc<dyn AccountService>>)
                        -> actix_web::Result<impl Responder> {
    let account_id = data.create_account(&request.0).await?;
    Ok(web::Json(account_id))
}


pub fn init_routes(config: &mut web::ServiceConfig, account_service: Arc<dyn AccountService>) {
    let data = Data::new(account_service);
    config.service(map_endpoints_to_functions().app_data(data));
}

fn map_endpoints_to_functions() -> Scope {
    web::scope("/account")
        .route("/id/{id}", web::get().to(get_account_by_id))
        .route("/create", web::post().to(create_account))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{App, test};
    use uuid::Uuid;

    use crate::accounting::account::account_http_api::map_endpoints_to_functions;
    use crate::accounting::account::account_models::{Account, CreateAccountRequest};
    use crate::accounting::account::account_service::{AccountService, MockAccountService};

    #[tokio::test]
    async fn test_api() {
        let mut mocked = MockAccountService::new();
        mocked.expect_create_account().returning(|_a| Ok(Default::default()));
        mocked.expect_get_account_by_id().returning(|_| Ok(Some(Default::default())));
        let mock: Arc<dyn AccountService> = Arc::new(mocked);
        let uuid = Uuid::now_v7();
        let account_expected = mock.get_account_by_id(&uuid).await.unwrap().unwrap();
        let app_data = actix_web::web::Data::new(mock);
        let app = App::new()
            .service(map_endpoints_to_functions())
            .app_data(app_data);
        let app_service = test::init_service(app).await;
        let uri = format!("/account/id/{}", uuid);
        let request = test::TestRequest::get()
            .uri(uri.as_str())
            .to_request();
        let res: Account = test::call_and_read_body_json(&app_service, request).await;
        assert_eq!(res, account_expected);
        let request = test::TestRequest::post()
            .uri("/account/create")
            .set_json(CreateAccountRequest { ..Default::default() })
            .to_request();
        let _: Uuid = test::call_and_read_body_json(&app_service, request).await;
    }
}

