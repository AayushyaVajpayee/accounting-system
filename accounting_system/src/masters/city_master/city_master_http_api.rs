use std::sync::Arc;
use actix_web::web::{Data, ServiceConfig};
use actix_web::{web, Scope};
use crate::masters::city_master::city_master_service::CityMasterService;

pub fn init_routes(config: &mut ServiceConfig, city_master_service: Arc<dyn CityMasterService>) {
    let data = Data::new(city_master_service);
    config.service(map_endpoints_to_functions().app_data(data));
}

fn map_endpoints_to_functions() -> Scope {
    web::scope("/city-master")
}
