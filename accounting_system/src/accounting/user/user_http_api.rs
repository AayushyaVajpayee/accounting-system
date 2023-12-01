use std::sync::Arc;

use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use uuid::Uuid;

use crate::accounting::user::user_models::CreateUserRequest;
use crate::accounting::user::user_service::{UserService, UserServiceError};

impl ResponseError for UserServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            UserServiceError::Db(_) => { StatusCode::INTERNAL_SERVER_ERROR }
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            UserServiceError::Db(err) => {
                HttpResponse::build(self.status_code()).json(err.to_string())
            }
        }
    }
}

async fn get_user_by_id(id: web::Path<Uuid>, data: web::Data<Arc<dyn UserService>>) -> actix_web::Result<impl Responder> {
    let p = data.get_user_by_id(id.into_inner()).await?;
    Ok(web::Json(p))
}

async fn create_user(request: web::Json<CreateUserRequest>,
                     data: Data<Arc<dyn UserService>>) -> actix_web::Result<impl Responder> {
    let p = data.create_user(&request.0).await?;
    Ok(web::Json(p))
}

pub fn init_routes(config: &mut web::ServiceConfig, user_service: Arc<dyn UserService>) {
    let data = Data::new(user_service);
    config.service(
        map_endpoints_to_functions().app_data(data)
    );
}

fn map_endpoints_to_functions() -> Scope {
    web::scope("user")
        .route("/id/{id}", web::get().to(get_user_by_id))
        .route("/create", web::post().to(create_user))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{App, test};
    use uuid::Uuid;

    use crate::accounting::user::user_http_api::map_endpoints_to_functions;
    use crate::accounting::user::user_models::{SEED_USER_ID, User};
    use crate::accounting::user::user_models::tests::{a_create_user_request, a_user, UserTestDataBuilder};
    use crate::accounting::user::user_service::{MockUserService, UserService};

    #[tokio::test]
    async fn test_api() {
        let mut mocked = MockUserService::new();
        mocked.expect_get_user_by_id().returning(|_| Ok(Some(a_user(
            UserTestDataBuilder { id: Some(Default::default()), ..Default::default() }
        ))));
        mocked.expect_create_user().returning(|_| Ok(Default::default()));
        let mock: Arc<dyn UserService> = Arc::new(mocked);
        let exp = mock.get_user_by_id(*SEED_USER_ID).await.unwrap().unwrap();
        let exp1 = mock.create_user(&a_create_user_request(Default::default())).await.unwrap();
        let app_data = actix_web::web::Data::new(mock);
        let app = App::new()
            .service(map_endpoints_to_functions())
            .app_data(app_data);
        let app = test::init_service(app).await;
        let uri = format!("/user/id/{}", *SEED_USER_ID);
        let req = test::TestRequest::get()
            .uri(uri.as_str())
            .to_request();
        let res: User = test::call_and_read_body_json(&app, req).await;
        assert_eq!(exp, res);
        let req = test::TestRequest::post()
            .uri("/user/create")
            .set_json(a_create_user_request(Default::default()))
            .to_request();
        let res: Uuid = test::call_and_read_body_json(&app, req).await;
        assert_eq!(exp1, res);
    }
}