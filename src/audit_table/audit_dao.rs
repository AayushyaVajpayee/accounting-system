use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::Pool;
use tokio_postgres::Row;
use uuid::Uuid;
use crate::audit_table::audit_model::AuditEntry;

#[async_trait]
pub trait AuditDao {
    async fn get_audit_logs_for_id_and_table(&self, id: Uuid, table_name: &str) -> Vec<AuditEntry>;
}


const SELECT_FIELDS: &str = "id,tenant_id,audit_record_id,operation_type,old_record,table_id,created_at";

const TABLE_NAME: &str = "audit_entries";

const QUERY_BY_TABLE_AND_ID: &str = concatcp!("select ",SELECT_FIELDS," from ",TABLE_NAME," ae join pg_class pc on pc.oid=ae.table_id  where pc.relname=$1 and ae.audit_record_id=$2");

pub fn get_audit_dao(client: &'static Pool) -> Box<dyn AuditDao + Send + Sync> {
    let audit_dao = AuditDaoImpl {
        postgres_client: client
    };
    Box::new(audit_dao)
}

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
    use deadpool_postgres::GenericClient;
    use uuid::Uuid;
    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::audit_table::audit_dao::{AuditDao, AuditDaoImpl};
    use crate::audit_table::audit_model::AuditEntry;

    #[tokio::test]
    async fn test() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let audit_dao = AuditDaoImpl { postgres_client };
        {
            let conn = audit_dao.postgres_client.get().await.unwrap();
            let raw_script = r#"
            create table test_audit_trigger(
                id uuid primary key,
                tenant_id serial,
                name varchar(40),
                created_at bigint default extract(epoch from now()) *1000000
            );

            create trigger audit_test_audit_trigger
            after update or delete on test_audit_trigger
            for each row
            execute function create_audit_entry();
            insert into test_audit_trigger(id,name) values(uuid_generate_v7(),'something');
            update test_audit_trigger set name='something updated';
        "#;
            conn.batch_execute(raw_script).await.unwrap();
        }
        let entry_id: Option<Uuid>;
        {
            let conn = audit_dao.postgres_client.get().await.unwrap();
            let all_entries = conn.query("select * from audit_entries", &[])
                .await.unwrap().iter()
                .map(|row| row.try_into().unwrap())
                .collect::<Vec<AuditEntry>>();
            entry_id = Some(all_entries
                .first()
                .unwrap()
                .audit_record_id);
        }
        let oho = audit_dao
            .get_audit_logs_for_id_and_table(entry_id.unwrap(), "test_audit_trigger")
            .await;
        assert_eq!(oho.len(), 1);
    }
}