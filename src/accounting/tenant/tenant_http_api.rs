use actix_web::{Responder, web};
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
    config.service(web::scope("/tenant")
        .app_data(data)
        .route("/id/{id}", web::get().to(get_tenant_by_id)))
        .route("/create", web::post().to(create_tenant));
}