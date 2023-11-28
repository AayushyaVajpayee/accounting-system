use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use crate::accounting::account::account_dao::{AccountDao, get_account_dao};
use crate::accounting::account::account_models::{Account, CreateAccountRequest};
use crate::accounting::postgres_factory::get_postgres_conn_pool;
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait AccountService:Send+Sync {
    async fn get_account_by_id(&self, id: &Uuid) -> Option<Account>;
    async fn create_account(&self, request: &CreateAccountRequest) -> Uuid;
}

struct AccountServiceImpl {
    account_dao: Arc<dyn AccountDao>,
}

#[async_trait]
impl AccountService for AccountServiceImpl {
    async fn get_account_by_id(&self, id: &Uuid) -> Option<Account> {
        self.account_dao.get_account_by_id(id).await
    }

    async fn create_account(&self, request: &CreateAccountRequest) -> Uuid {
        self.account_dao.create_account(request).await
    }
}

pub fn get_account_service() -> Arc<dyn AccountService> {
    let pool = get_postgres_conn_pool();
    let dao = get_account_dao(pool);
    let service = AccountServiceImpl { account_dao: dao };
    Arc::new(service)
}
#[cfg(test)]
pub fn get_account_service_for_test(client: &'static deadpool_postgres::Pool) -> Arc<dyn AccountService> {
    Arc::new(AccountServiceImpl { account_dao: get_account_dao(client) })
}