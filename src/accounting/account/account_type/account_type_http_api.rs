use std::sync::Arc;

use actix_web::{Scope, web};
use actix_web::web::{Data, ServiceConfig};
use crate::accounting::account::account_type::account_type_service::AccountTypeService;
pub fn init_routes(config: &mut ServiceConfig, account_type_service: Arc<dyn AccountTypeService>) {
    let data = Data::new(account_type_service);
    config.service(map_endpoints_to_functions().app_data(data));
}

fn map_endpoints_to_functions() -> Scope {
    web::scope("/account-type-master")
}
