use std::sync::Arc;

use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web::{Data, Path};

use crate::tenant::tenant_models::CreateTenantRequest;
use crate::tenant::tenant_service::{TenantService, TenantServiceError};

async fn get_tenant_by_id(
    id: Path<Uuid>,
    data: Data<Arc<dyn TenantService>>,
) -> actix_web::Result<impl Responder> {
    let t = data.get_tenant_by_id(id.into_inner()).await?;
    Ok(web::Json(t))
}

//todo need to write a test for this. How?
async fn create_tenant(
    request: web::Json<CreateTenantRequest>,
    data: Data<Arc<dyn TenantService>>,
) -> actix_web::Result<impl Responder> {
    let p = data.create_tenant(&request.0).await?;
    Ok(web::Json(p))
}

#[derive(Serialize,Debug)]
struct Errors<'a> {
    errors: &'a Vec<String>,
}
#[derive(Deserialize,Debug)]
struct ErrorsResponse{
    errors:Vec<String>
}
impl ResponseError for TenantServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            TenantServiceError::Validation(_) => StatusCode::BAD_REQUEST,
            TenantServiceError::DB(_) | TenantServiceError::Other(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self{
            TenantServiceError::Validation(errs) => {
                HttpResponse::build(self.status_code()).json(Errors { errors: errs })
            }
            TenantServiceError::DB(errs) => {
                let err_list = vec![errs.to_string()];
                HttpResponse::build(self.status_code()).json(Errors { errors: &err_list })
            }
            TenantServiceError::Other(a) => {
                let err_list = vec![self.to_string()];
                HttpResponse::build(self.status_code()).json(Errors { errors: &err_list })
            }
        }
    }
}
pub fn init_routes(config: &mut web::ServiceConfig, tenant_service: Arc<dyn TenantService>) {
    let data = Data::new(tenant_service);
    config.service(map_endpoints_to_functions().app_data(data));
}

fn map_endpoints_to_functions() -> Scope {
    web::scope("/tenant")
        .route("/id/{id}", web::get().to(get_tenant_by_id))
        .route("/create", web::post().to(create_tenant))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{App, test};
    use uuid::Uuid;

    use crate::tenant::tenant_http_api::map_endpoints_to_functions;
    use crate::tenant::tenant_models::{a_create_tenant_request, CreateTenantTestBuilder, SEED_TENANT_ID, Tenant};
    use crate::tenant::tenant_service::{MockTenantService, TenantService};

    #[tokio::test]
    async fn test_api() {
        let mut mocked = MockTenantService::new();
        mocked.expect_get_tenant_by_id()
            .returning(|_|Ok(Some(Default::default())));
        mocked.expect_create_tenant()
            .returning(|_|Ok(Default::default()));
        let mock: Arc<dyn TenantService> = Arc::new(mocked);
        let tenant_expected = mock.get_tenant_by_id(*SEED_TENANT_ID).await.unwrap();
        let app_data = actix_web::web::Data::new(mock);
        let app = App::new()
            .service(map_endpoints_to_functions())
            .app_data(app_data);
        let app_service = test::init_service(app).await;
        let uri = format!("/tenant/id/{}", *SEED_TENANT_ID);
        let request = test::TestRequest::get().uri(&uri).to_request();
        let res: Tenant = test::call_and_read_body_json(&app_service, request).await;
        assert_eq!(res, tenant_expected.unwrap());
        let request = test::TestRequest::post()
            .uri("/tenant/create")
            .set_json(
                a_create_tenant_request(CreateTenantTestBuilder{..Default::default()}))
            .to_request();
        let _: Uuid = test::call_and_read_body_json(&app_service, request).await;
    }
}
