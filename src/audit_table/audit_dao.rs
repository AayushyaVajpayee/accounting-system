use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::Pool;
use tokio_postgres::Row;
use uuid::Uuid;
use crate::audit_table::audit_model::AuditEntry;

#[async_trait]
trait AuditDao {
    async fn get_audit_logs_for_id_and_table(&self, id: Uuid, table_name: &str) -> Vec<AuditEntry>;
}


const SELECT_FIELDS: &str = "id,tenant_id,audit_record_id,operation_type,old_record,table_id,created_at";

const TABLE_NAME: &str = "audit_entries";

const QUERY_BY_TABLE_AND_ID: &str = concatcp!("select ",SELECT_FIELDS," from ",TABLE_NAME," where table_id=$1 and audit_record_id=$2");

impl TryFrom<&Row> for AuditEntry {
    type Error = ();

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(AuditEntry {
            id: row.get(0),
            tenant_id: row.get(1),
            audit_record_id: row.get(2),
            operation_type: row.get(3),
            old_record: row.get(4),
            table_id: row.get(5),
            created_at: row.get(6),
        })
    }
}

struct AuditDaoImpl {
    postgres_client: &'static Pool,
}

#[async_trait]
impl AuditDao for AuditDaoImpl {
    async fn get_audit_logs_for_id_and_table(&self, id: Uuid, table_name: &str) -> Vec<AuditEntry> {
        let conn = self.postgres_client.get().await.unwrap();
        conn.query(
            QUERY_BY_TABLE_AND_ID,
            &[&table_name, &id],
        ).await.unwrap()
            .iter().map(|row| row.try_into().unwrap())
            .collect::<Vec<AuditEntry>>()
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test() {}
}