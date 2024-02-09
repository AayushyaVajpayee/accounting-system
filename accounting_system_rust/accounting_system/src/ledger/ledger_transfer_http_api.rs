use actix_web::{ web};
use actix_web::web::Data;

use crate::ledger::ledger_transfer_service::LedgerTransferService;
use crate::setup_routes;

setup_routes!(LedgerTransferService,"/ledger-transfer",);
