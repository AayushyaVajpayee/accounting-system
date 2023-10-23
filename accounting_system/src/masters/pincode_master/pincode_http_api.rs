use std::sync::Arc;
use actix_web::Scope;
use actix_web::web::{Data, ServiceConfig};
use crate::masters::pincode_master::pincode_master_service::PincodeMasterService;

pub fn init_routes(config: &mut ServiceConfig, pincode_master_service: Arc<dyn PincodeMasterService>) {
    let data = Data::new(pincode_master_service);
    config.service(map_endpoints_to_functions().app_data(data));
}

fn map_endpoints_to_functions() -> Scope {
    actix_web::web::scope("/pincode-master")
}
