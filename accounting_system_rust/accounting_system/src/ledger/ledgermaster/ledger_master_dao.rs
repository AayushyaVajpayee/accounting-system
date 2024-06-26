use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use deadpool_postgres::Pool;
use tokio_postgres::Row;
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::ledger::ledgermaster::ledger_master_models::{
    CreateLedgerMasterEntryRequest, LedgerMaster,
};

#[async_trait]
pub trait LedgerMasterDao: Send + Sync {
    async fn get_ledger_master_by_id(&self, id: &Uuid) -> Option<LedgerMaster>;
    async fn create_ledger_master_entry(&self, request: &CreateLedgerMasterEntryRequest) -> Uuid;
}

#[allow(dead_code)]
pub fn get_ledger_master_dao(client: Arc<Pool>) -> Arc<dyn LedgerMasterDao> {
    Arc::new(LedgerMasterPostgresDaoImpl {
        postgres_client: client,
    })
}

struct LedgerMasterPostgresDaoImpl {
    postgres_client: Arc<Pool>,
}

const SELECT_FIELDS: &str = "id,tenant_id,display_name,\
currency_master_id,created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "ledger_master";
static BY_ID_QUERY: OnceLock<String> = OnceLock::new();
static INSERT_STATEMENT: OnceLock<String> = OnceLock::new();

impl TryFrom<&Row> for LedgerMaster {
    type Error = ();

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(LedgerMaster {
            id: row.get(0),
            tenant_id: row.get(1),
            display_name: row.get(2),
            currency_master_id: row.get(3),
            audit_metadata: AuditMetadataBase {
                created_by: row.get(4),
                updated_by: row.get(5),
                created_at: row.get(6),
                updated_at: row.get(7),
            },
        })
    }
}

impl LedgerMasterPostgresDaoImpl {
    fn get_ledger_master_entry_by_id_query() -> &'static String {
        BY_ID_QUERY
            .get_or_init(|| format!("select {} from {} where id=$1", SELECT_FIELDS, TABLE_NAME))
    }
    fn create_insert_statement() -> &'static String {
        INSERT_STATEMENT.get_or_init(|| {
            format!(
                "insert into {} ({}) values\
             ($1,$2,$3,$4,$5,$6,$7,$8) returning id",
                TABLE_NAME, SELECT_FIELDS
            )
        })
    }
}

#[async_trait]
impl LedgerMasterDao for LedgerMasterPostgresDaoImpl {
    async fn get_ledger_master_by_id(&self, id: &Uuid) -> Option<LedgerMaster> {
        let query = LedgerMasterPostgresDaoImpl::get_ledger_master_entry_by_id_query();
        let conn = self.postgres_client.get().await.unwrap();
        let values = conn.query(query, &[&id]).await.unwrap();
        values.iter().map(|row| row.try_into().unwrap()).next()
    }

    async fn create_ledger_master_entry(&self, request: &CreateLedgerMasterEntryRequest) -> Uuid {
        let query = LedgerMasterPostgresDaoImpl::create_insert_statement();
        let conn = self.postgres_client.get().await.unwrap();
        let id = Uuid::now_v7();
        conn.query(
            query,
            &[
                &id,
                &request.tenant_id,
                &request.display_name,
                &request.currency_master_id,
                &request.audit_metadata.created_by,
                &request.audit_metadata.updated_by,
                &request.audit_metadata.created_at,
                &request.audit_metadata.updated_at,
            ],
        )
        .await
        .unwrap()
        .iter()
        .map(|row| row.get(0))
        .next()
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::accounting::postgres_factory::test_utils_postgres::{
        get_dao_generic, get_postgres_conn_pool, get_postgres_image_port,
    };
    use crate::ledger::ledgermaster::ledger_master_dao::{
        LedgerMasterDao, LedgerMasterPostgresDaoImpl,
    };
    use crate::ledger::ledgermaster::ledger_master_models::tests::a_create_ledger_master_entry_request;

    #[tokio::test]
    async fn should_be_able_to_create_and_fetch_ledger_master() {
        let ledger_master_dao = get_dao_generic(
            |a| LedgerMasterPostgresDaoImpl {
                postgres_client: a.clone(),
            },
            None,
        )
        .await;
        let ledger_master = a_create_ledger_master_entry_request(Default::default());
        let id = ledger_master_dao
            .create_ledger_master_entry(&ledger_master)
            .await;
        let fetched_ledger_master = ledger_master_dao
            .get_ledger_master_by_id(&id)
            .await
            .unwrap();
        assert_eq!(fetched_ledger_master.id, id);
    }
}
