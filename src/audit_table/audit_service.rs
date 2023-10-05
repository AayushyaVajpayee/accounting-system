use async_trait::async_trait;
use uuid::Uuid;
use crate::audit_table::audit_model::AuditEntry;

//this is only for reading
#[async_trait]
pub trait AuditService {
    async fn get_audit_logs_for_id_and_table(&self, id: Uuid, table_name: &str) -> Vec<AuditEntry>;
}

struct AuditServiceImpl {}