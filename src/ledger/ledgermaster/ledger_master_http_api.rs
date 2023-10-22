use std::sync::Arc;

use actix_web::{Scope, web};
use actix_web::web::{Data, ServiceConfig};

use crate::ledger::ledgermaster::ledger_master_service::LedgerMasterService;

pub fn init_routes(config: &mut ServiceConfig, ledger_master_service: Arc<dyn LedgerMasterService>) {
    let data = Data::new(ledger_master_service);
    config.service(map_endpoints_to_functions().app_data(data));
}

fn map_endpoints_to_functions() -> Scope {
    web::scope("/ledger-master")
}
