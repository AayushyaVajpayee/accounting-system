use actix_web::{Responder, web};
use crate::accounting::tenant::tenant_service::{get_tenant_service, TenantService};

async fn get_tenant_by_id(mut data: web::Data<Box<dyn TenantService + Send + Sync>>) -> actix_web::Result<impl Responder> {
    let t = data.get_tenant_by_id(&1).await;
    Ok(web::Json(t))
}


pub fn init_routes(config: &mut web::ServiceConfig) {
    let tenant_service = get_tenant_service();
    let data = web::Data::new(tenant_service);
    config.service(web::scope("/tenant")
        .app_data(data)
        .route("/id", web::get().to(get_tenant_by_id)))
    ;
}