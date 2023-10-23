use std::sync::Arc;
use crate::audit_table::audit_service::{AuditService};
use actix_web::web::{Data, ServiceConfig};
use actix_web::{web, Scope};

pub fn init_routes(config: &mut ServiceConfig, audit_table_service: Arc<dyn AuditService>) {
    let data = Data::new(audit_table_service);
    config.service(map_endpoints_to_functions().app_data(data));
}

fn map_endpoints_to_functions() -> Scope {
    web::scope("/audit-table")
}
