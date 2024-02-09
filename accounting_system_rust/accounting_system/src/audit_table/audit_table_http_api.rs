use actix_web::{ web};
use actix_web::web::Data;

use crate::audit_table::audit_service::AuditService;
use crate::setup_routes;

setup_routes!(AuditService,"/audit-table",);
