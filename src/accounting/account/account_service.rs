use async_trait::async_trait;
use deadpool_postgres::Pool;

use crate::accounting::account::account_dao::{AccountDao, get_account_dao};
use crate::accounting::account::account_models::{Account, CreateAccountRequest};

#[async_trait]
pub trait AccountService {
    async fn get_account_by_id(&self, id: &i32) -> Option<Account>;
    async fn create_account(&self, request: &CreateAccountRequest) -> i32;
}

struct AccountServiceImpl {
    account_dao: Box<dyn AccountDao + Send + Sync>,
}

#[async_trait]
impl AccountService for AccountServiceImpl {
    async fn get_account_by_id(&self, id: &i32) -> Option<Account> {
        self.account_dao.get_account_by_id(id).await
    }

    async fn create_account(&self, request: &CreateAccountRequest) -> i32 {
        self.account_dao.create_account(request).await
    }
}

#[cfg(test)]
pub fn get_account_service_for_test(client: &'static Pool) -> Box<dyn AccountService + Send + Sync> {
    Box::new(AccountServiceImpl { account_dao: get_account_dao(client) })
}