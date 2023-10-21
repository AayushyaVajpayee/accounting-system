use std::sync::Arc;
use actix_web::{Responder, Scope, web};
use actix_web::web::{Data, Path};

use crate::accounting::account::account_models::CreateAccountRequest;
use crate::accounting::account::account_service::{AccountService, get_account_service};

async fn get_account_by_id(id: Path<i32>,
                           data: Data<Arc<dyn AccountService>>)
                           -> actix_web::Result<impl Responder> {
    let account = data.get_account_by_id(&id).await;
    Ok(web::Json(account))
}


async fn create_account(request: web::Json<CreateAccountRequest>,
                        data: Data<Arc<dyn AccountService>>)
                        -> actix_web::Result<impl Responder> {
    let account_id = data.create_account(&request.0).await;
    Ok(web::Json(account_id))
}


pub fn init_routes(config: &mut web::ServiceConfig) {
    let account_service = get_account_service();
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
    use async_trait::async_trait;

    use crate::accounting::account::account_http_api::map_endpoints_to_functions;
    use crate::accounting::account::account_models::{Account, CreateAccountRequest};
    use crate::accounting::account::account_service::AccountService;

    struct MockAccountService {}

    #[async_trait]
    impl AccountService for MockAccountService {
        async fn get_account_by_id(&self, _id: &i32) -> Option<Account> {
            Some(Default::default())
        }

        async fn create_account(&self, _request: &CreateAccountRequest) -> i32 {
            0
        }
    }

    #[tokio::test]
    async fn test_api() {
        let mock: Arc<dyn AccountService> = Arc::new(MockAccountService {});
        let tenant_expected = mock.get_account_by_id(&1).await.unwrap();
        let app_data = actix_web::web::Data::new(mock);
        let app = App::new()
            .service(map_endpoints_to_functions())
            .app_data(app_data);
        let app_service = test::init_service(app).await;
        let request = test::TestRequest::get()
            .uri("/account/id/1")
            .to_request();
        let res: Account = test::call_and_read_body_json(&app_service, request).await;
        assert_eq!(res, tenant_expected);
        let request = test::TestRequest::post()
            .uri("/account/create")
            .set_json(Account { ..Default::default() })
            .to_request();
        let _: i32 = test::call_and_read_body_json(&app_service, request).await;
    }
}

