use std::sync::Arc;
use crate::audit_table::audit_service::{AuditService};
use actix_web::web::{Data};
use actix_web::{web, Scope};
use crate::setup_routes;


setup_routes!(AuditService,"/audit-table",);
