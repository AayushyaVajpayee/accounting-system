use std::sync::Arc;

use actix_web::{Scope, web};
use actix_web::web::{Data, ServiceConfig};

use crate::ledger::ledger_transfer_service::LedgerTransferService;

pub fn init_routes(config: &mut ServiceConfig, ledger_transfer_service: Arc<dyn LedgerTransferService>) {
    let data = Data::new(ledger_transfer_service);
    config.service(map_endpoints_to_functions().app_data(data));
}

fn map_endpoints_to_functions() -> Scope {
    web::scope("/ledger-transfer")
}
