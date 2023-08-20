use actix_web::{Responder, Scope, web};
use web::{Data, Path};
use crate::accounting::tenant::tenant_models::CreateTenantRequest;

use crate::accounting::tenant::tenant_service::{get_tenant_service, TenantService};

async fn get_tenant_by_id(id: Path<i32>,
                          data: Data<Box<dyn TenantService + Send + Sync>>)
                          -> actix_web::Result<impl Responder> {
    let t = data.get_tenant_by_id(&id).await;
    Ok(web::Json(t))
}


//todo need to write a test for this. How?
async fn create_tenant(request: web::Json<CreateTenantRequest>,
                       data: Data<Box<dyn TenantService + Send + Sync>>)
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
    use actix_web::{App, test};
    use async_trait::async_trait;
    use crate::accounting::tenant::tenant_http_api::map_endpoints_to_functions;
    use crate::accounting::tenant::tenant_models::{CreateTenantRequest, Tenant};
    use crate::accounting::tenant::tenant_service::TenantService;

    struct MockTenantService {}

    #[async_trait]
    impl TenantService for MockTenantService {
        async fn get_tenant_by_id(&self, id: &i32) -> Option<Tenant> {
            Some(Default::default())
        }

        async fn create_tenant(&self, tenant: &CreateTenantRequest) -> i32 {
            0
        }
    }

    #[tokio::test]
    async fn test_api() {
        let mock: Box<dyn TenantService + Send + Sync> = Box::new(MockTenantService {});
        let tenant_expected = mock.get_tenant_by_id(&1).await.unwrap();
        let app_data = actix_web::web::Data::new(mock);
        let app = App::new()
            .service(map_endpoints_to_functions())
            .app_data(app_data);
        let app_service = test::init_service(app).await;
        let request = test::TestRequest::get()
            .uri("/tenant/id/1")
            .to_request();
        let res: Tenant = test::call_and_read_body_json(&app_service, request).await;
        assert_eq!(res, tenant_expected);
        let request = test::TestRequest::post()
            .uri("/tenant/create")
            .set_json(Tenant { ..Default::default() })
            .to_request();
        let _: i32 = test::call_and_read_body_json(&app_service, request).await;
    }
}