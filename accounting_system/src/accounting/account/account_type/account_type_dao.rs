use std::sync::Arc;

use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::Pool;
use tokio_postgres::Row;
use uuid::Uuid;

use crate::accounting::account::account_type::account_type_models::{
    AccountTypeMaster, CreateAccountTypeMasterRequest,
};
use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::common_utils::dao_error::DaoError;

const SELECT_FIELDS: &str =
    "id,tenant_id,child_ids,parent_id,display_name,account_code,created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "account_type_master";
const BY_ID_QUERY: &str = concatcp!(
    "select ",
    SELECT_FIELDS,
    " from ",
    TABLE_NAME,
    " where id=$1"
);

const INSERT_STATEMENT: &str = concatcp!(
    "insert into ",
    TABLE_NAME,
    " (",
    SELECT_FIELDS,
    ")",
    " values ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) returning id"
);
const ALL_TYPES_FOR_TENANT: &str = concatcp!(
    "select ",
    SELECT_FIELDS,
    " from ",
    TABLE_NAME,
    " where tenant_id=$1"
);

#[async_trait]
pub trait AccountTypeDao: Send + Sync {
    async fn get_account_type_by_id(&self, id: &Uuid)
        -> Result<Option<AccountTypeMaster>, DaoError>;
    async fn create_account_type(
        &self,
        request: &CreateAccountTypeMasterRequest,
    ) -> Result<Uuid, DaoError>;
    async fn get_all_account_types_for_tenant_id(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<AccountTypeMaster>, DaoError>;
}

struct AccountTypeDaoPostgresImpl {
    postgres_client: &'static Pool,
}

impl TryFrom<&Row> for AccountTypeMaster {
    type Error = DaoError;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(AccountTypeMaster {
            id: row.get(0),
            tenant_id: row.get(1),
            child_ids: row.try_get(2).ok(),
            parent_id: row.try_get(3).ok(),
            display_name: row.get(4),
            account_code: row.get(5),
            audit_metadata: AuditMetadataBase {
                created_by: row.get(6),
                updated_by: row.get(7),
                created_at: row.get(8),
                updated_at: row.get(9),
            },
        })
    }
}

#[async_trait]
impl AccountTypeDao for AccountTypeDaoPostgresImpl {
    async fn get_account_type_by_id(
        &self,
        id: &Uuid,
    ) -> Result<Option<AccountTypeMaster>, DaoError> {
        let query = BY_ID_QUERY;
        let db_rows = self
            .postgres_client
            .get()
            .await?
            .query(query, &[id])
            .await?;
        let account_type = db_rows
            .iter()
            .map(|row| row.try_into())
            .next()
            .transpose()?;
        Ok(account_type)
    }

    async fn create_account_type(
        &self,
        request: &CreateAccountTypeMasterRequest,
    ) -> Result<Uuid, DaoError> {
        let query = INSERT_STATEMENT;
        let account_type_id: Option<Uuid> = self
            .postgres_client
            .get()
            .await?
            .query(
                query,
                &[
                    &request.tenant_id,
                    &request.child_ids,
                    &request.parent_id,
                    &request.display_name,
                    &request.account_code,
                    &request.audit_metadata.created_by,
                    &request.audit_metadata.updated_by,
                    &request.audit_metadata.created_at,
                    &request.audit_metadata.updated_at,
                ],
            )
            .await?
            .iter()
            .map(|row| row.get(0))
            .next();
        account_type_id.ok_or_else(|| {
            DaoError::PostgresQueryError("no id returned by insert statement".to_string())
        })
    }

    async fn get_all_account_types_for_tenant_id(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<AccountTypeMaster>, DaoError> {
        let query = ALL_TYPES_FOR_TENANT;
       let account_types = self.postgres_client
            .get()
            .await?
            .query(query, &[&tenant_id])
            .await?
            .iter()
            .map(|row| row.try_into())
            .collect();
        account_types
    }
}

pub fn get_account_type_dao(pool: &'static Pool) -> Arc<dyn AccountTypeDao> {
    let dao = AccountTypeDaoPostgresImpl {
        postgres_client: pool,
    };
    Arc::new(dao)
}
#[cfg(test)]
mod account_type_tests {
    use crate::accounting::account::account_type::account_type_dao::{
        AccountTypeDao, AccountTypeDaoPostgresImpl,
    };
    use crate::accounting::account::account_type::account_type_models::{
        a_create_account_type_master_request, CreateAccountTypeMasterRequestTestBuilder,
    };
    use crate::accounting::postgres_factory::test_utils_postgres::{
        get_postgres_conn_pool, get_postgres_image_port,
    };
    use crate::tenant::tenant_models::SEED_TENANT_ID;

    #[tokio::test]
    async fn tests() {
        let port = get_postgres_image_port().await;
        let account_type_dao = AccountTypeDaoPostgresImpl {
            postgres_client: get_postgres_conn_pool(port).await,
        };
        let an_account_type =
            a_create_account_type_master_request(CreateAccountTypeMasterRequestTestBuilder {
                tenant_id: Some(*SEED_TENANT_ID),
                ..Default::default()
            });
        let account_type_id = account_type_dao.create_account_type(&an_account_type).await.unwrap();
        let _ = account_type_dao
            .get_account_type_by_id(&account_type_id)
            .await
            .unwrap();
        let k = account_type_dao
            .get_all_account_types_for_tenant_id(*SEED_TENANT_ID)
            .await.unwrap();
        assert!(k.len() > 5);
    }
}
