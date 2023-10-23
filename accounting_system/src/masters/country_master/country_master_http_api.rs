use std::sync::Arc;
use actix_web::web::{Data, ServiceConfig};
use actix_web::{web, Scope};
use crate::masters::country_master::country_service::CountryMasterService;

pub fn init_routes(config: &mut ServiceConfig, country_master_service: Arc<dyn CountryMasterService>) {
    let data = Data::new(country_master_service);
    config.service(map_endpoints_to_functions().app_data(data));
}

fn map_endpoints_to_functions() -> Scope {
    web::scope("/country-master")
}
