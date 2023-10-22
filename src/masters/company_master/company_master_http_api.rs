use std::sync::Arc;
use actix_web::{web, Responder, Scope};
use actix_web::web::{Data, ServiceConfig};
use crate::masters::company_master::company_master_service::CompanyMasterService;
//
// async fn get_company_by_id() -> actix_web::Result<impl Responder> {
//     todo!()
// }
//
// async fn create_company(
//     request: web::Json<CreateCompanyRequest>,
//     data: Data<Arc<dyn CompanyMasterService>>,
// ) -> actix_web::Result<impl Responder> {
//     todo!()
// }


pub fn init_routes(config: &mut ServiceConfig, country_master_service: Arc<dyn CompanyMasterService>) {
    let data = Data::new(country_master_service);
    config.service(map_endpoints_to_functions().app_data(data));
}

fn map_endpoints_to_functions() -> Scope {
    web::scope("/company-master")
}
