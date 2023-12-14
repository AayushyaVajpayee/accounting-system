use std::sync::Arc;
use actix_web::web::{Data, ServiceConfig};
use actix_web::{web, Scope};
use crate::masters::state_master::state_master_service::StateMasterService;
use crate::setup_routes;

setup_routes!(StateMasterService,"/state-master",);
