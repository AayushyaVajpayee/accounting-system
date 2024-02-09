use async_trait::async_trait;
use deadpool_postgres::Pool;
use std::sync::Arc;
use uuid::Uuid;

use crate::audit_table::audit_dao::{AuditDao, get_audit_dao};
use crate::audit_table::audit_model::AuditEntry;

//this is only for reading
#[async_trait]
pub trait AuditService:Send+Sync {
    async fn get_audit_logs_for_id_and_table(&self, id: Uuid, table_name: &str) -> Vec<AuditEntry>;
}

struct AuditServiceImpl {
    audit_dao: Arc<dyn AuditDao>,
}

#[allow(dead_code)]
pub fn get_audit_service(arc: Arc<Pool>) -> Arc<dyn AuditService> {
    let audit_dao = get_audit_dao(arc);
    let audit_service = AuditServiceImpl {
        audit_dao
    };
    Arc::new(audit_service)
}

#[cfg(test)]
pub fn get_audit_service_for_tests(pool: Arc<Pool>) -> Arc<dyn AuditService> {
    let audit_dao = get_audit_dao(pool);
    let audit_service = AuditServiceImpl {
        audit_dao
    };
    Arc::new(audit_service)
}
#[async_trait]
impl AuditService for AuditServiceImpl {
    async fn get_audit_logs_for_id_and_table(&self, id: Uuid, table_name: &str) -> Vec<AuditEntry> {
        self.audit_dao.get_audit_logs_for_id_and_table(id, table_name).await
    }
}