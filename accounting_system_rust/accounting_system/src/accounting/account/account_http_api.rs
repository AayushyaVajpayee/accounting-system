use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use std::sync::Arc;
use uuid::Uuid;

use crate::accounting::account::account_models::CreateAccountRequest;
use crate::accounting::account::account_service::{AccountService, AccountServiceError};
use crate::setup_routes;

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


setup_routes!(AccountService,
    "/account","/id/{id}",web::get().to(get_account_by_id),
    "/create",web::post().to(create_account));

#[cfg(test)]
mod tests {
    use actix_web::{App, test};
    use std::sync::Arc;
    use uuid::Uuid;

    use crate::accounting::account::account_http_api::map_endpoints_to_functions;
    use crate::accounting::account::account_models::{Account, CreateAccountRequest};
    use crate::accounting::account::account_service::{AccountService, MockAccountService};
    use crate::get_and_create_api_test;

    #[tokio::test]
    async fn test_api() {
        let closure=||{
        let mut mocked = MockAccountService::new();
        mocked.expect_create_account().returning(|_a| Ok(Default::default()));
        mocked.expect_get_account_by_id().returning(|_| Ok(Some(Default::default())));
            mocked
        };
        let uuid = Uuid::now_v7();
        let get_uri = format!("/account/id/{}", uuid);
        let account_expected:Account = Default::default();
        get_and_create_api_test!(Account,AccountService,closure,get_uri,
            "/account/create",CreateAccountRequest { ..Default::default() },account_expected);
    }
}

