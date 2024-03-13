use actix_web::web;
use actix_web::web::Data;

use crate::masters::pincode_master::pincode_master_service::PincodeMasterService;
use crate::setup_routes;

setup_routes!(PincodeMasterService,"/pincode-master",);
