use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::Pool;
use futures_util::TryFutureExt;
use std::sync::Arc;
use tokio_postgres::Row;
use uuid::Uuid;

use crate::accounting::currency::currency_models::{AuditMetadataBase, CreateCurrencyMasterRequest, CurrencyMaster};
use crate::common_utils::dao_error::DaoError;
use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_uuid;

const SELECT_FIELDS: &str = "id,tenant_id,scale,display_name,description,\
created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "currency_master";

const BY_ID_QUERY: &str = concatcp!("select ",SELECT_FIELDS," from ",TABLE_NAME," where id=$1");

const INSERT_STATEMENT: &str = concatcp!("insert into ",TABLE_NAME," (",SELECT_FIELDS,")"," values ($1,$2,$3,$4,$5,$6,$7,$8,$9) returning id");

#[async_trait]
pub trait CurrencyDao: Send + Sync {
    async fn get_currency_entry_by_id(&self, id: &Uuid) -> Result<Option<CurrencyMaster>, DaoError>;
    async fn create_currency_entry(&self, currency: &CreateCurrencyMasterRequest) -> Result<Uuid, DaoError>;
}

pub struct CurrencyDaoPostgresImpl {
    postgres_client: Arc<Pool>,
}

impl TryFrom<&Row> for CurrencyMaster {
    type Error = DaoError;

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

pub fn get_currency_dao(client: Arc<Pool>) -> Arc<dyn CurrencyDao> {
    let currency_dao = CurrencyDaoPostgresImpl {
        postgres_client: client
    };
    Arc::new(currency_dao)
}

#[async_trait]
impl CurrencyDao for CurrencyDaoPostgresImpl {
    async fn get_currency_entry_by_id(&self, id: &Uuid) -> Result<Option<CurrencyMaster>, DaoError> {
        let conn = self.postgres_client.get().await?;
        let k = conn.
            query(BY_ID_QUERY, &[id]).await?;
        let ans = k.iter().map(|row|
            row.try_into()).next().transpose()?;
        Ok(ans)
    }

    async fn create_currency_entry(&self, req: &CreateCurrencyMasterRequest) -> Result<Uuid, DaoError> {
        let simple_query = format!(r#"
        begin transaction;
        select create_currency(Row('{}','{}',{}::smallint,'{}','{}','{}','{}',{},{}));
        commit;
        "#,
                                   req.idempotence_key,
                                   req.tenant_id,
                                   req.scale,
                                   req.display_name,
                                   req.description,
                                   req.audit_metadata.created_by,
                                   req.audit_metadata.updated_by,
                                   req.audit_metadata.created_at,
                                   req.audit_metadata.updated_at);
        let conn = self.postgres_client.get().await?;
        let rows = conn.simple_query(simple_query.as_str()).await?;
        parse_db_output_of_insert_create_and_return_uuid(&rows)
    }
}

#[cfg(test)]
mod tests {
    use spectral::assert_that;
    use spectral::prelude::OptionAssertions;

    use crate::accounting::currency::currency_dao::{CurrencyDao, CurrencyDaoPostgresImpl};
    use crate::accounting::currency::currency_models::{a_create_currency_master_request, CreateCurrencyMasterRequestTestBuilder};
    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::tenant::tenant_models::SEED_TENANT_ID;

    #[tokio::test]
    async fn should_be_able_to_create_and_fetch_currency() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let currency_master = a_create_currency_master_request(
            CreateCurrencyMasterRequestTestBuilder {
                tenant_id: Some(*SEED_TENANT_ID),
                ..Default::default()
            }
        );
        let currency_dao = CurrencyDaoPostgresImpl { postgres_client: postgres_client.clone() };
        let curr_id = currency_dao.create_currency_entry(&currency_master).await.unwrap();
        let fetched_curr = currency_dao.get_currency_entry_by_id(&curr_id).await.unwrap().unwrap();
        assert_eq!(curr_id, fetched_curr.id)
    }


    #[tokio::test]
    async fn should_create_account_when_only_1_new_request() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let currency_request = a_create_currency_master_request(Default::default());
        let currency_dao = CurrencyDaoPostgresImpl { postgres_client: postgres_client.clone() };
        let id = currency_dao.create_currency_entry(&currency_request).await.unwrap();
        let curr = currency_dao.get_currency_entry_by_id(&id).await.unwrap();
        assert_that!(curr).is_some();
    }

    #[tokio::test]
    async fn should_return_existing_account_when_idempotency_key_is_same_as_earlier_completed_request() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let name = "tsting";
        let currency_request =
            a_create_currency_master_request(
                CreateCurrencyMasterRequestTestBuilder {
                    display_name: Some(name.to_string()),
                    ..Default::default()
                });
        let currency_dao = CurrencyDaoPostgresImpl { postgres_client: postgres_client.clone() };
        let id = currency_dao.create_currency_entry(&currency_request).await.unwrap();
        let id2 = currency_dao.create_currency_entry(&currency_request).await.unwrap();
        assert_that!(&id).is_equal_to(id2);
        let number_of_currs_created: i64 = postgres_client
            .get()
            .await
            .unwrap()
            .query(
                "select count(id) from currency_master where display_name=$1",
                &[&name],
            )
            .await
            .unwrap()
            .iter()
            .map(|a| a.get(0))
            .next()
            .unwrap();
        assert_that!(number_of_currs_created).is_equal_to(1)
        ;
    }
}