use std::sync::Arc;
use actix_web::web::{Data, ServiceConfig};
use actix_web::{web, Scope};
use crate::masters::country_master::country_service::CountryMasterService;
use crate::setup_routes;

setup_routes!(CountryMasterService,"/country-master",);
