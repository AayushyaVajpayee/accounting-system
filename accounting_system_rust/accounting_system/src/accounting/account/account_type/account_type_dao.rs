use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::Pool;
use itertools::Itertools;
use std::sync::Arc;
use tokio_postgres::Row;
use uuid::Uuid;

use crate::accounting::account::account_type::account_type_models::{
    AccountTypeMaster, CreateAccountTypeMasterRequest,
};
use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::common_utils::dao_error::DaoError;
use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_uuid;

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

const ALL_TYPES_FOR_TENANT: &str = concatcp!(
    "select ",
    SELECT_FIELDS,
    " from ",
    TABLE_NAME,
    " where tenant_id=$1"
);

#[async_trait]
pub trait AccountTypeDao: Send + Sync {
    async fn get_account_type_by_id(
        &self,
        id: &Uuid,
    ) -> Result<Option<AccountTypeMaster>, DaoError>;
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
    postgres_client: Arc<Pool>,
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
        let simple_query = format!(
            r#"
           begin transaction;
           select create_account_type_master(Row('{}','{}',{},{},'{}',{},'{}','{}',{},{}));
           commit;
        "#,
            request.idempotence_key,
            request.tenant_id,
            request
                .child_ids
                .as_ref()
                .map(|a| {
                    let k = a.iter().map(|a| format!("'{}'", a)).join(",");
                    format!("ARRAY[{}]", k)
                })
                .unwrap_or_else(|| "null".to_string()),
            request
                .parent_id
                .map(|a| format!("'{}'", a))
                .unwrap_or_else(|| "null".to_string()),
            request.display_name,
            request
                .account_code
                .map(|a| a.to_string())
                .unwrap_or_else(|| "null".to_string()),
            request.audit_metadata.created_by,
            request.audit_metadata.updated_by,
            request.audit_metadata.created_at,
            request.audit_metadata.updated_at
        );
        let conn = self.postgres_client.get().await?;
        let rows = conn.simple_query(simple_query.as_str()).await?;
        parse_db_output_of_insert_create_and_return_uuid(&rows)
    }

    async fn get_all_account_types_for_tenant_id(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<AccountTypeMaster>, DaoError> {
        let query = ALL_TYPES_FOR_TENANT;
        let account_types = self
            .postgres_client
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

#[allow(dead_code)]
pub fn get_account_type_dao(pool: Arc<Pool>) -> Arc<dyn AccountTypeDao> {
    let dao = AccountTypeDaoPostgresImpl {
        postgres_client: pool,
    };
    Arc::new(dao)
}

#[cfg(test)]
mod account_type_tests {
    use speculoos::assert_that;
    use speculoos::prelude::OptionAssertions;

    use crate::accounting::account::account_type::account_type_dao::{
        AccountTypeDao, AccountTypeDaoPostgresImpl,
    };
    use crate::accounting::account::account_type::account_type_models::tests::a_create_account_type_master_request;
    use crate::accounting::account::account_type::account_type_models::CreateAccountTypeMasterRequestBuilder;
    use crate::accounting::postgres_factory::test_utils_postgres::{
        get_dao_generic, get_postgres_conn_pool, get_postgres_image_port,
    };
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    #[tokio::test]
    async fn tests() {
        let account_type_dao = get_dao_generic(
            |a| AccountTypeDaoPostgresImpl {
                postgres_client: a.clone(),
            },
            None,
        )
        .await;
        let an_account_type = a_create_account_type_master_request(Default::default());
        let account_type_id = account_type_dao
            .create_account_type(&an_account_type)
            .await
            .unwrap();
        let _ = account_type_dao
            .get_account_type_by_id(&account_type_id)
            .await
            .unwrap();
        let k = account_type_dao
            .get_all_account_types_for_tenant_id(*SEED_TENANT_ID)
            .await
            .unwrap();
        assert!(k.len() > 5);
    }

    #[tokio::test]
    async fn should_create_account_type_mst_when_only_1_new_request() {
        let account_type_mst_request = a_create_account_type_master_request(Default::default());
        let account_type_dao = get_dao_generic(
            |a| AccountTypeDaoPostgresImpl {
                postgres_client: a.clone(),
            },
            None,
        )
        .await;
        let id = account_type_dao
            .create_account_type(&account_type_mst_request)
            .await
            .unwrap();
        let acc = account_type_dao.get_account_type_by_id(&id).await.unwrap();
        assert_that!(acc).is_some();
    }

    #[tokio::test]
    async fn should_return_existing_account_type_when_idempotency_key_is_same_as_earlier_completed_request(
    ) {
        let account_type_dao = get_dao_generic(
            |a| AccountTypeDaoPostgresImpl {
                postgres_client: a.clone(),
            },
            None,
        )
        .await;
        let name = "tsting";
        let mut builder = CreateAccountTypeMasterRequestBuilder::default();
        builder.display_name(name.to_string());
        let account_type_mst_request = a_create_account_type_master_request(builder);
        let id = account_type_dao
            .create_account_type(&account_type_mst_request)
            .await
            .unwrap();
        let id2 = account_type_dao
            .create_account_type(&account_type_mst_request)
            .await
            .unwrap();
        assert_that!(&id).is_equal_to(id2);
        let number_of_acc_types_created: i64 = account_type_dao
            .postgres_client
            .get()
            .await
            .unwrap()
            .query(
                "select count(id) from account_type_master where display_name=$1",
                &[&name],
            )
            .await
            .unwrap()
            .iter()
            .map(|a| a.get(0))
            .next()
            .unwrap();
        assert_that!(number_of_acc_types_created).is_equal_to(1);
    }
}
