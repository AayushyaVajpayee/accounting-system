use std::sync::OnceLock;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use tokio_postgres::Row;


use crate::accounting::account::account_models::{Account, CreateAccountRequest};
use crate::accounting::currency::currency_models::AuditMetadataBase;

const ACCOUNT_POSTGRES_SELECT_FIELDS: &str = "id,tenant_id,display_code,account_type_id,\
user_id,ledger_master_id,debits_posted,debits_pending,credits_posted,\
credits_pending,created_by,updated_by,created_at,updated_at";
const ACCOUNT_TABLE_NAME: &str = "user_account";
static ACCOUNT_BY_ID_QUERY: OnceLock<String> = OnceLock::new();
static ACCOUNT_INSERT_STATEMENT: OnceLock<String> = OnceLock::new();

#[async_trait]
pub trait AccountDao {
    async fn get_account_by_id(&self, id: &i32) -> Option<Account>;
    async fn create_account(&self, request: &CreateAccountRequest) -> i32;
}

pub struct AccountDaoPostgresImpl {
    postgres_client: &'static Pool,
}

impl TryFrom<&Row> for Account {
    type Error = ();

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(
            Account {
                id: row.get(0),
                tenant_id: row.get(1),
                display_code: row.get(2),
                account_type_id: row.get(3),
                user_id: row.get(4),
                ledger_master_id: row.get(5),
                debits_posted: row.get(6),
                debits_pending: row.get(7),
                credits_posted: row.get(8),
                credits_pending: row.get(9),
                audit_metadata: AuditMetadataBase {
                    created_by: row.get(10),
                    updated_by: row.get(11),
                    created_at: row.get(12),
                    updated_at: row.get(13),
                },
            }
        )
    }
}

pub fn get_account_dao(client: &'static Pool) -> Box<dyn AccountDao + Send + Sync> {
    Box::new(AccountDaoPostgresImpl {
        postgres_client: client
    })
}

impl AccountDaoPostgresImpl {
    fn get_account_by_id_query() -> &'static str {
        ACCOUNT_BY_ID_QUERY.get_or_init(|| {
            format!("select {ACCOUNT_POSTGRES_SELECT_FIELDS} from {ACCOUNT_TABLE_NAME} where id=$1")
        })
    }

    fn get_insert_statement() -> &'static str {
        ACCOUNT_INSERT_STATEMENT.get_or_init(|| {
            format!("insert into {ACCOUNT_TABLE_NAME} ({ACCOUNT_POSTGRES_SELECT_FIELDS}) values\
            (DEFAULT,$1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) returning id")
        })
    }
}

#[async_trait]
impl AccountDao for AccountDaoPostgresImpl {
    async fn get_account_by_id(&self, id: &i32) -> Option<Account> {
        let query = AccountDaoPostgresImpl::get_account_by_id_query();
        let k = self.postgres_client.get().await.unwrap().query(
            query,
            &[id],
        ).await.unwrap();
        k.iter()
            .map(|row|
                row.try_into().unwrap()
            ).next()
    }

    async fn create_account(&self, request: &CreateAccountRequest) -> i32 {
        let query = AccountDaoPostgresImpl::get_insert_statement();
        self.postgres_client.get().await.unwrap().query(
            query,
            &[
                &request.tenant_id,
                &request.display_code,
                &request.account_type_id,
                &request.user_id,
                &request.ledger_master_id,
                &0i64, &0i64, &0i64, &0i64,
                &request.audit_metadata.created_by,
                &request.audit_metadata.updated_by,
                &request.audit_metadata.created_at,
                &request.audit_metadata.updated_at
            ],
        ).await
            .unwrap()
            .iter()
            .map(|row| row.get(0))
            .next()
            .unwrap()
    }
}


#[cfg(test)]
mod account_tests {
    use crate::accounting::account::account_dao::{AccountDao, AccountDaoPostgresImpl};
    use crate::accounting::account::account_models::{a_create_account_request, CreateAccountRequestTestBuilder};
    use crate::test_utils::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};

    #[tokio::test]
    async fn test_account() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let an_account_request = a_create_account_request(CreateAccountRequestTestBuilder {
            tenant_id: Some(1),
            ledger_master_id: Some(1),
            account_type_id: Some(1),
            user_id: Some(1),
            ..Default::default()
        });
        let mut account_dao = AccountDaoPostgresImpl { postgres_client };
        let account_id = account_dao.create_account(&an_account_request).await;
        let account_fetched = account_dao.get_account_by_id(&account_id).await.unwrap();
        assert_eq!(account_fetched.id, account_id)
    }
}