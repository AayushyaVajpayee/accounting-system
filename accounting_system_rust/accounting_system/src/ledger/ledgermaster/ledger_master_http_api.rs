use actix_web::{ web};
use actix_web::web::{Data};

use crate::ledger::ledgermaster::ledger_master_service::LedgerMasterService;
use crate::setup_routes;

setup_routes!(LedgerMasterService,"ledger-master",);
