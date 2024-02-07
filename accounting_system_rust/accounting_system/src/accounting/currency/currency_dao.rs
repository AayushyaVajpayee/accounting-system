use std::sync::Arc;

use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::Pool;
use tokio_postgres::Row;
use uuid::Uuid;

use crate::accounting::currency::currency_models::{CreateCurrencyMasterRequest, CurrencyMaster};
use crate::common_utils::dao_error::DaoError;
use crate::common_utils::db_row_conversion_utils::{convert_row_to_audit_metadata_base, convert_row_to_base_master_fields};
use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_uuid;

const SELECT_FIELDS: &str = "id,entity_version_id,tenant_id,active,approval_status,remarks,scale,display_name,description,\
created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "currency_master";

const BY_ID_QUERY: &str = concatcp!("select ",SELECT_FIELDS," from ",TABLE_NAME," where id=$1 and tenant_id=$2");

const INSERT_STATEMENT: &str = concatcp!("insert into ",TABLE_NAME," (",SELECT_FIELDS,")"," values ($1,$2,$3,$4,$5,$6,$7,$8,$9) returning id");

#[async_trait]
pub trait CurrencyDao: Send + Sync {
    async fn get_currency_entry_by_id(&self, id: Uuid,tenant_id:Uuid) -> Result<Option<CurrencyMaster>, DaoError>;
    async fn create_currency_entry(&self, currency: &CreateCurrencyMasterRequest,tenant_id:Uuid) -> Result<Uuid, DaoError>;
}

pub struct CurrencyDaoPostgresImpl {
    postgres_client: Arc<Pool>,
}

impl TryFrom<&Row> for CurrencyMaster {
    type Error = DaoError;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let (base_master_fields,next_ind) =  convert_row_to_base_master_fields(row)?;
        Ok(CurrencyMaster {
           base_master_fields,
            scale: row.get(next_ind),
            display_name: row.get(next_ind+1),
            description: row.get(next_ind+2),
            audit_metadata: convert_row_to_audit_metadata_base(next_ind+3,row)?,
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
    async fn get_currency_entry_by_id(&self, id: Uuid,tenant_id:Uuid) -> Result<Option<CurrencyMaster>, DaoError> {
        let conn = self.postgres_client.get().await?;
        let k = conn.
            query(BY_ID_QUERY, &[&id,&tenant_id]).await?;
        let ans = k.iter().map(|row|
            row.try_into()).next().transpose()?;
        Ok(ans)
    }

    async fn create_currency_entry(&self, req: &CreateCurrencyMasterRequest,tenant_id:Uuid) -> Result<Uuid, DaoError> {
        let simple_query = format!(r#"
        begin transaction;
        select create_currency(Row('{}','{}',{}::smallint,'{}','{}','{}','{}',{},{}));
        commit;
        "#,
                                   req.idempotence_key,
                                   tenant_id,
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
    use crate::accounting::currency::currency_models::CreateCurrencyMasterRequestBuilder;
    use crate::accounting::currency::currency_models::tests::a_create_currency_master_request;
    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    #[tokio::test]
    async fn should_be_able_to_create_and_fetch_currency() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let currency_master = a_create_currency_master_request(Default::default());
        let currency_dao = CurrencyDaoPostgresImpl { postgres_client: postgres_client.clone() };
        let curr_id = currency_dao.create_currency_entry(&currency_master,*SEED_TENANT_ID).await.unwrap();
        let fetched_curr = currency_dao.get_currency_entry_by_id(curr_id,*SEED_TENANT_ID).await.unwrap().unwrap();
        assert_eq!(curr_id, fetched_curr.base_master_fields.id)
    }


    #[tokio::test]
    async fn should_create_account_when_only_1_new_request() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let currency_request = a_create_currency_master_request(Default::default());
        let currency_dao = CurrencyDaoPostgresImpl { postgres_client: postgres_client.clone() };
        let id = currency_dao.create_currency_entry(&currency_request,*SEED_TENANT_ID).await.unwrap();
        let curr = currency_dao.get_currency_entry_by_id(id,*SEED_TENANT_ID).await.unwrap();
        assert_that!(curr).is_some();
    }

    #[tokio::test]
    async fn should_return_existing_account_when_idempotency_key_is_same_as_earlier_completed_request() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let name = "tsting";
        let mut builder = CreateCurrencyMasterRequestBuilder::default();
        builder.display_name(name.to_string());
        let currency_request =
            a_create_currency_master_request(builder);
        let currency_dao = CurrencyDaoPostgresImpl { postgres_client: postgres_client.clone() };
        let id = currency_dao.create_currency_entry(&currency_request,*SEED_TENANT_ID).await.unwrap();
        let id2 = currency_dao.create_currency_entry(&currency_request,*SEED_TENANT_ID).await.unwrap();
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