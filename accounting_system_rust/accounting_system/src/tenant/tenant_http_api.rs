use std::sync::Arc;

use actix_web::{HttpResponse, Responder, ResponseError, web};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web::{Data, Path};

use crate::setup_routes;
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
            TenantServiceError::Db(_) | TenantServiceError::Other(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self{
            TenantServiceError::Validation(errs) => {
                HttpResponse::build(self.status_code()).json(Errors { errors: errs })
            }
            TenantServiceError::Db(errs) => {
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

setup_routes!(TenantService,
    "/tenant","/id/{id}",web::get().to(get_tenant_by_id),
    "/create",web::post().to(create_tenant));

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::test;

    use crate::get_and_create_api_test;
    use crate::tenant::tenant_http_api::map_endpoints_to_functions;
    use crate::tenant::tenant_models::{ SEED_TENANT_ID, Tenant};
    use crate::tenant::tenant_models::tests::a_create_tenant_request;
    use crate::tenant::tenant_service::{MockTenantService, TenantService};

    #[tokio::test]
    async fn test_api() {
        let closure =||{
            let mut mocked = MockTenantService::new();
            mocked.expect_get_tenant_by_id()
                .returning(|_| Ok(Some(Default::default())));
            mocked.expect_create_tenant()
                .returning(|_| Ok(Default::default()));
            mocked
        };
        let get_uri=format!("/tenant/id/{}", *SEED_TENANT_ID);
        let tenant_expected:Tenant = Default::default();
        get_and_create_api_test!(Tenant,TenantService,closure,
            get_uri,"/tenant/create",
            a_create_tenant_request(Default::default()),tenant_expected);
    }
}
