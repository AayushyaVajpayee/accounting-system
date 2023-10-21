use std::sync::Arc;
use actix_web::{Responder, Scope, web};
use uuid::Uuid;
use web::{Data, Path};

use crate::tenant::tenant_models::CreateTenantRequest;
use crate::tenant::tenant_service::{get_tenant_service, TenantService};

async fn get_tenant_by_id(id: Path<Uuid>,
                          data: Data<Arc<dyn TenantService>>)
                          -> actix_web::Result<impl Responder> {
    let t = data.get_tenant_by_id(id.into_inner()).await;
    Ok(web::Json(t))
}


//todo need to write a test for this. How?
async fn create_tenant(request: web::Json<CreateTenantRequest>,
                       data: Data<Arc<dyn TenantService>>)
                       -> actix_web::Result<impl Responder> {
    let p = data.create_tenant(&request.0).await;
    Ok(web::Json(p))
}


pub fn init_routes(config: &mut web::ServiceConfig) {
    let tenant_service = get_tenant_service();
    let data = Data::new(tenant_service);
    config.service(
        map_endpoints_to_functions()
            .app_data(data)
    );
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
    use async_trait::async_trait;
    use uuid::Uuid;

    use crate::tenant::tenant_http_api::map_endpoints_to_functions;
    use crate::tenant::tenant_models::{CreateTenantRequest, SEED_TENANT_ID, Tenant};
    use crate::tenant::tenant_service::TenantService;

    struct MockTenantService {}

    #[async_trait]
    impl TenantService for MockTenantService {
        async fn get_tenant_by_id(&self, _id: Uuid) -> Option<Tenant> {
            Some(Default::default())
        }

        async fn create_tenant(&self, _tenant: &CreateTenantRequest) -> Uuid {

            *SEED_TENANT_ID
        }
    }

    #[tokio::test]
    async fn test_api() {
        let mock: Arc<dyn TenantService> = Arc::new(MockTenantService {});
        let tenant_expected = mock.get_tenant_by_id(*SEED_TENANT_ID).await.unwrap();
        let app_data = actix_web::web::Data::new(mock);
        let app = App::new()
            .service(map_endpoints_to_functions())
            .app_data(app_data);
        let app_service = test::init_service(app).await;
        let uri = format!("/tenant/id/{}",*SEED_TENANT_ID);
        let request = test::TestRequest::get()
            .uri(&uri)
            .to_request();
        let res: Tenant = test::call_and_read_body_json(&app_service, request).await;
        assert_eq!(res, tenant_expected);
        let request = test::TestRequest::post()
            .uri("/tenant/create")
            .set_json(Tenant { ..Default::default() })
            .to_request();
        let _: Uuid = test::call_and_read_body_json(&app_service, request).await;
    }
}