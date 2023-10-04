use actix_web::{Responder, Scope, web};
use actix_web::web::Data;

use crate::accounting::user::user_models::CreateUserRequest;
use crate::accounting::user::user_service::{get_user_service, UserService};

async fn get_user_by_id(id: web::Path<i32>, data: web::Data<Box<dyn UserService + Send + Sync>>) -> actix_web::Result<impl Responder> {
    let p = data.get_user_by_id(&id).await;
    Ok(web::Json(p))
}

async fn create_user(request: web::Json<CreateUserRequest>,
                     data: Data<Box<dyn UserService + Send + Sync>>) -> actix_web::Result<impl Responder> {
    let p = data.create_user(&request.0).await;
    Ok(web::Json(p))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    let user_service = get_user_service();
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
    use actix_web::{App, test};
    use async_trait::async_trait;

    use crate::accounting::currency::currency_models::AuditMetadataBase;
    use crate::accounting::user::user_http_api::map_endpoints_to_functions;
    use crate::accounting::user::user_models::{a_create_user_request, CreateUserRequest, User};
    use crate::accounting::user::user_service::UserService;

    struct UserServiceMock {}

    #[async_trait]
    impl UserService for UserServiceMock {
        async fn get_user_by_id(&self, _id: &i32) -> Option<User> {
            Some(User {
                id:0,
                tenant_id: 0,
                first_name: "a".to_string(),
                last_name: None,
                email_id: None,
                mobile_number: None,
                audit_metadata: AuditMetadataBase {
                    created_by: "".to_string(),
                    updated_by: "".to_string(),
                    created_at: 0,
                    updated_at: 0,
                },
            })
        }

        async fn create_user(&self, _user: &CreateUserRequest) -> i32 {
            1
        }
    }

    #[tokio::test]
    async fn test_api() {
        let mock: Box<dyn UserService + Send + Sync> = Box::new(UserServiceMock {});
        let exp = mock.get_user_by_id(&7).await.unwrap();
        let exp1 = mock.create_user(&a_create_user_request(Default::default())).await;
        let app_data = actix_web::web::Data::new(mock);
        let app = App::new()
            .service(map_endpoints_to_functions())
            .app_data(app_data);
        let app = test::init_service(app).await;
        let req = test::TestRequest::get()
            .uri("/user/id/1")
            .to_request();
        let res: User = test::call_and_read_body_json(&app, req).await;
        assert_eq!(exp, res);
        let req = test::TestRequest::post()
            .uri("/user/create")
            .set_json(a_create_user_request(Default::default()))
            .to_request();
        let res: i32 = test::call_and_read_body_json(&app, req).await;
        assert_eq!(exp1, res);
    }
}