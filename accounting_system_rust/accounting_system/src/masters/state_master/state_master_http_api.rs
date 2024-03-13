use actix_web::web;
use actix_web::web::Data;

use crate::masters::state_master::state_master_service::StateMasterService;
use crate::setup_routes;

setup_routes!(StateMasterService,"/state-master",);
