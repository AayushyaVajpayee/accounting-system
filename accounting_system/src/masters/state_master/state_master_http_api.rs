use std::sync::Arc;
use actix_web::web::{Data, ServiceConfig};
use actix_web::{web, Scope};
use crate::masters::state_master::state_master_service::StateMasterService;

pub fn init_routes(config: &mut ServiceConfig, state_master_service: Arc<dyn StateMasterService>) {
    let data = Data::new(state_master_service);
    config.service(map_endpoints_to_functions().app_data(data));
}

fn map_endpoints_to_functions() -> Scope {
    web::scope("/state-master")
}
