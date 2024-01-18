use actix_web::{Scope, web};
use actix_web::web::{Data, ServiceConfig};
use std::sync::Arc;

use crate::masters::country_master::country_service::CountryMasterService;
use crate::setup_routes;

setup_routes!(CountryMasterService,"/country-master",);
