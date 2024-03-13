use actix_web::web;
use actix_web::web::Data;

use crate::masters::city_master::city_master_service::CityMasterService;
use crate::setup_routes;

setup_routes!(CityMasterService, "/city-master",);
