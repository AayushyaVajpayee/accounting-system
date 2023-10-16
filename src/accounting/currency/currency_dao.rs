use std::sync::OnceLock;

use async_trait::async_trait;
use deadpool_postgres::Pool;
use tokio_postgres::Row;

use crate::accounting::currency::currency_models::{AuditMetadataBase, CreateCurrencyMasterRequest, CurrencyMaster};

const SELECT_FIELDS: &str = "id,tenant_id,scale,display_name,description,\
created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "currency_master";
static BY_ID_QUERY: OnceLock<String> = OnceLock::new();
static INSERT_STATEMENT: OnceLock<String> = OnceLock::new();

#[async_trait]
pub trait CurrencyDao {
    async fn get_currency_entry_by_id(&self, id: &i16) -> Option<CurrencyMaster>;
    async fn create_currency_entry(&self, currency: &CreateCurrencyMasterRequest) -> i16;
}

pub struct CurrencyDaoPostgresImpl {
    postgres_client: &'static Pool,
}

impl TryFrom<&Row> for CurrencyMaster {
    type Error = ();

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(CurrencyMaster {
            id: row.get(0),
            tenant_id: row.get(1),
            scale: row.get(2),
            display_name: row.get(3),
            description: row.get(4),
            audit_metadata: AuditMetadataBase {
                created_by: row.get(5),
                updated_by: row.get(6),
                created_at: row.get(7),
                updated_at: row.get(8),
            },
        })
    }
}

impl CurrencyDaoPostgresImpl {
    fn get_currency_master_by_id_query() -> &'static String {
        BY_ID_QUERY.get_or_init(|| {
            format!("select {} from {} where id=$1", SELECT_FIELDS, TABLE_NAME)
        })
    }
    fn create_insert_statement() -> &'static String {
        INSERT_STATEMENT.get_or_init(|| {
            format!("insert into {} ({}) values \
            (DEFAULT,$1,$2,$3,$4,$5,$6,$7,$8) returning id",
                    TABLE_NAME,
                    SELECT_FIELDS)
        })
    }
}

pub fn get_currency_dao(client: &'static Pool) -> Box<dyn CurrencyDao + Send + Sync> {
    let currency_dao = CurrencyDaoPostgresImpl {
        postgres_client: client
    };
    Box::new(currency_dao)
}

#[async_trait]
impl CurrencyDao for CurrencyDaoPostgresImpl {
    async fn get_currency_entry_by_id(&self, id: &i16) -> Option<CurrencyMaster> {
        let query = CurrencyDaoPostgresImpl::get_currency_master_by_id_query();
        let conn = self.postgres_client.get().await.unwrap();
        let k = conn.
            query(query, &[id]).await.unwrap();
        k.iter().map(|row|
            row.try_into().unwrap()).next()
    }

    async fn create_currency_entry(&self, currency: &CreateCurrencyMasterRequest) -> i16 {
        let query = CurrencyDaoPostgresImpl::create_insert_statement();
        let conn = self.postgres_client.get().await.unwrap();
        conn.query(
            query,
            &[&(currency.tenant_id), &(currency.scale), &currency.display_name,
                &currency.description,
                &currency.audit_metadata.created_by, &currency.audit_metadata.updated_by,
                &(currency.audit_metadata.created_at), &(currency.audit_metadata.updated_at)],
        ).await.unwrap().iter().map(|row| row.get(0)).next().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::accounting::currency::currency_dao::{CurrencyDao, CurrencyDaoPostgresImpl};
    use crate::accounting::currency::currency_models::{a_create_currency_master_request, CreateCurrencyMasterRequestTestBuilder};
    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::tenant::tenant_models::SEED_TENANT_ID;

    #[tokio::test]
    async fn should_be_able_to_create_and_fetch_currency() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let currency_master = a_create_currency_master_request(
            CreateCurrencyMasterRequestTestBuilder {
                tenant_id: Some(*SEED_TENANT_ID),
                ..Default::default()
            }
        );
        let  currency_dao = CurrencyDaoPostgresImpl { postgres_client };
        let curr_id = currency_dao.create_currency_entry(&currency_master).await;
        let fetched_curr = currency_dao.get_currency_entry_by_id(&curr_id).await.unwrap();
        assert_eq!(curr_id, fetched_curr.id)
    }
}