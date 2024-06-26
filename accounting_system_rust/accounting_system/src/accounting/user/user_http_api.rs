use std::sync::Arc;

use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder, ResponseError};
use uuid::Uuid;

use crate::accounting::user::user_models::CreateUserRequest;
use crate::accounting::user::user_service::{UserService, UserServiceError};
use crate::common_utils::utils::{TenantId, UserId};
use crate::setup_routes;

impl ResponseError for UserServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            UserServiceError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
            UserServiceError::Validation(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            UserServiceError::Db(err) => {
                HttpResponse::build(self.status_code()).json(err.to_string())
            }
            UserServiceError::Validation(_) => {
                HttpResponse::build(self.status_code()).json(self.to_string())
            }
        }
    }
}

async fn get_user_by_id(
    id: web::Path<Uuid>,
    data: Data<Arc<dyn UserService>>,
    tenant_id: TenantId,
) -> actix_web::Result<impl Responder> {
    let p = data
        .get_user_by_id(id.into_inner(), tenant_id.inner())
        .await?;
    Ok(web::Json(p))
}

async fn create_user(
    request: web::Json<CreateUserRequest>,
    data: Data<Arc<dyn UserService>>,
    tenant_id: TenantId,
    user_id: UserId,
) -> actix_web::Result<impl Responder> {
    let p = data
        .create_user(&request.0, tenant_id.inner(), user_id.inner())
        .await?;
    Ok(web::Json(p))
}

setup_routes!(
    UserService,
    "user",
    "/id/{id}",
    web::get().to(get_user_by_id),
    "/create",
    web::post().to(create_user)
);

#[cfg(test)]
mod tests {
    use crate::accounting::user::user_http_api::map_endpoints_to_functions;
    use crate::accounting::user::user_models::tests::{
        a_create_user_request, a_user, UserTestDataBuilder,
    };
    use crate::accounting::user::user_models::User;
    use crate::accounting::user::user_service::{MockUserService, UserService};
    use crate::get_and_create_api_test_v2;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    #[tokio::test]
    async fn test_api() {
        let closure = || {
            let mut mocked = MockUserService::new();
            mocked.expect_get_user_by_id().returning(|_, _| {
                Ok(Some(Arc::new(a_user(UserTestDataBuilder {
                    id: Some(Default::default()),
                    ..Default::default()
                }))))
            });
            mocked
                .expect_create_user()
                .returning(|_, _, _| Ok(Default::default()));
            mocked
        };

        let get_uri = format!("/user/id/{}", *SEED_USER_ID);
        let exp_user = a_user(UserTestDataBuilder {
            id: Some(Default::default()),
            ..Default::default()
        });
        get_and_create_api_test_v2!(
            User,
            UserService,
            closure,
            get_uri,
            "/user/create",
            a_create_user_request(Default::default()),
            exp_user,
            *SEED_TENANT_ID
        );
    }
}
