use std::sync::Arc;

use actix_web::{Scope, web};
use actix_web::web::{Data};
use crate::accounting::account::account_type::account_type_service::AccountTypeService;
use crate::setup_routes;


setup_routes!(AccountTypeService,
    "/account-type-master",);

