use std::sync::OnceLock;

use async_trait::async_trait;
use deadpool_postgres::Pool;
use tokio_postgres::Row;
use uuid::Uuid;

use crate::accounting::account::account_type::account_type_models::{AccountTypeMaster, CreateAccountTypeMasterRequest};
use crate::accounting::currency::currency_models::AuditMetadataBase;

const SELECT_FIELDS: &str =
    "id,tenant_id,child_ids,parent_id,display_name,account_code,created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "account_type_master";
static BY_ID_QUERY: OnceLock<String> = OnceLock::new();
static INSERT_STATEMENT: OnceLock<String> = OnceLock::new();
static ALL_TYPES_FOR_TENANT: OnceLock<String> = OnceLock::new();

#[async_trait]
pub trait AccountTypeDao {
    async fn get_account_type_by_id(&self, id: &i16) -> Option<AccountTypeMaster>;
    async fn create_account_type(&self, request: &CreateAccountTypeMasterRequest) -> i16;
    async fn get_all_account_types_for_tenant_id(&self, tenant_id: Uuid) -> Vec<AccountTypeMaster>;
}

pub struct AccountTypeDaoPostgresImpl {
    postgres_client: &'static Pool,
}

impl AccountTypeDaoPostgresImpl {
    fn get_account_type_by_id_query() -> &'static String {
        BY_ID_QUERY.get_or_init(|| {
            format!("select {SELECT_FIELDS} from {TABLE_NAME} where id=$1")
        })
    }

    fn create_insert_statement() -> &'static String {
        INSERT_STATEMENT.get_or_init(|| {
            format!("insert into {TABLE_NAME} ({SELECT_FIELDS})\
            values (DEFAULT,$1,$2,$3,$4,$5,$6,$7,$8,$9) returning id")
        })
    }

    fn get_all_types_for_tenant_id_query() -> &'static String {
        ALL_TYPES_FOR_TENANT.get_or_init(|| {
            format!("select {SELECT_FIELDS} from {TABLE_NAME} where tenant_id=$1")
        })
    }
}

impl TryFrom<&Row> for AccountTypeMaster {
    type Error = ();

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
    async fn get_account_type_by_id(&self, id: &i16) -> Option<AccountTypeMaster> {
        let query = AccountTypeDaoPostgresImpl::get_account_type_by_id_query();
        let k = self.postgres_client.get().await.unwrap()
            .query(query,
                   &[id]).await.unwrap();
        k.iter().map(|row|
            row.try_into().unwrap()
        ).next()
    }

    async fn create_account_type(&self, request: &CreateAccountTypeMasterRequest) -> i16 {
        let query = AccountTypeDaoPostgresImpl::create_insert_statement();
        self.postgres_client.get().await.unwrap().query(
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
                &request.audit_metadata.updated_at
            ],
        ).await.unwrap()
            .iter()
            .map(|row| row.get(0))
            .next()
            .unwrap()
    }

    async fn get_all_account_types_for_tenant_id(&self, tenant_id: Uuid) -> Vec<AccountTypeMaster> {
        let query = AccountTypeDaoPostgresImpl::get_all_types_for_tenant_id_query();
        self.postgres_client.get()
            .await.unwrap().query(query, &[&tenant_id]).await
            .unwrap().iter().map(|row| row.try_into().unwrap()).collect()
    }
}


#[cfg(test)]
mod account_type_tests {
    use crate::accounting::account::account_type::account_type_dao::{AccountTypeDao, AccountTypeDaoPostgresImpl};
    use crate::accounting::account::account_type::account_type_models::{a_create_account_type_master_request,
                                                                        CreateAccountTypeMasterRequestTestBuilder};
    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::tenant::tenant_models::SEED_TENANT_ID;

    #[tokio::test]
    async fn tests() {
        let port = get_postgres_image_port().await;
        let  account_type_dao = AccountTypeDaoPostgresImpl {
            postgres_client: get_postgres_conn_pool(port).await
        };
        let an_account_type = a_create_account_type_master_request(
            CreateAccountTypeMasterRequestTestBuilder {
                tenant_id: Some(*SEED_TENANT_ID),
                ..Default::default()
            });
        let account_type_id = account_type_dao
            .create_account_type(&an_account_type).await;
        let _ = account_type_dao
            .get_account_type_by_id(&account_type_id).await
            .unwrap();
        let k = account_type_dao
            .get_all_account_types_for_tenant_id(*SEED_TENANT_ID).await;
        assert!(k.len() > 5);
    }
}
