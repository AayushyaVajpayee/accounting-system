use std::sync::Arc;

use actix_web::{Scope, web};
use actix_web::web::{Data, ServiceConfig};

use crate::ledger::ledgermaster::ledger_master_service::LedgerMasterService;
use crate::setup_routes;


setup_routes!(LedgerMasterService,"ledger-master",);
