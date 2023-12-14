use std::sync::Arc;
use actix_web::Scope;
use actix_web::web;
use actix_web::web::{Data, ServiceConfig};
use crate::masters::pincode_master::pincode_master_service::PincodeMasterService;
use crate::setup_routes;

setup_routes!(PincodeMasterService,"/pincode-master",);
