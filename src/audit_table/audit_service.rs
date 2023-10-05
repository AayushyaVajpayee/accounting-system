use async_trait::async_trait;
use uuid::Uuid;
use crate::accounting::postgres_factory::get_postgres_conn_pool;
use crate::audit_table::audit_dao::{AuditDao, get_audit_dao};
use crate::audit_table::audit_model::AuditEntry;

//this is only for reading
#[async_trait]
pub trait AuditService {
    async fn get_audit_logs_for_id_and_table(&self, id: Uuid, table_name: &str) -> Vec<AuditEntry>;
}

struct AuditServiceImpl {
    audit_dao: Box<dyn AuditDao + Send + Sync>,
}

#[allow(dead_code)]
pub fn get_audit_service() -> Box<dyn AuditService + Send + Sync> {
    let pclient = get_postgres_conn_pool();
    let audit_dao = get_audit_dao(pclient);
    let audit_service = AuditServiceImpl {
        audit_dao
    };
    Box::new(audit_service)
}

#[async_trait]
impl AuditService for AuditServiceImpl {
    async fn get_audit_logs_for_id_and_table(&self, id: Uuid, table_name: &str) -> Vec<AuditEntry> {
        self.audit_dao.get_audit_logs_for_id_and_table(id, table_name).await
    }
}