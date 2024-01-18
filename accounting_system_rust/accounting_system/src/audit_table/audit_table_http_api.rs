use actix_web::{Scope, web};
use actix_web::web::Data;
use std::sync::Arc;

use crate::audit_table::audit_service::AuditService;
use crate::setup_routes;

setup_routes!(AuditService,"/audit-table",);
