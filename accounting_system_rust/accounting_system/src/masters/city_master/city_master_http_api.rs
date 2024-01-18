use actix_web::{Scope, web};
use actix_web::web::{Data, ServiceConfig};
use std::sync::Arc;

use crate::masters::city_master::city_master_service::CityMasterService;
use crate::setup_routes;

setup_routes!(CityMasterService, "/city-master",);
