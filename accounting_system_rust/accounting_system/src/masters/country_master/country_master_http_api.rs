use actix_web::{ web};
use actix_web::web::{Data };

use crate::masters::country_master::country_service::CountryMasterService;
use crate::setup_routes;

setup_routes!(CountryMasterService,"/country-master",);
