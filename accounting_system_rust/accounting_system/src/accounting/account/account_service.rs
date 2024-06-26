use async_trait::async_trait;
use deadpool_postgres::Pool;
#[cfg(test)]
use mockall::automock;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use crate::accounting::account::account_dao::{get_account_dao, AccountDao};
use crate::accounting::account::account_models::{Account, CreateAccountRequest};
use crate::common_utils::dao_error::DaoError;

#[derive(Debug, Error)]
pub enum AccountServiceError {
    #[error(transparent)]
    Db(#[from] DaoError),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait AccountService: Send + Sync {
    async fn get_account_by_id(&self, id: &Uuid) -> Result<Option<Account>, AccountServiceError>;
    async fn create_account(
        &self,
        request: &CreateAccountRequest,
    ) -> Result<Uuid, AccountServiceError>;
}

struct AccountServiceImpl {
    account_dao: Arc<dyn AccountDao>,
}

#[async_trait]
impl AccountService for AccountServiceImpl {
    async fn get_account_by_id(&self, id: &Uuid) -> Result<Option<Account>, AccountServiceError> {
        self.account_dao
            .get_account_by_id(id)
            .await
            .map_err(|a| a.into())
    }

    async fn create_account(
        &self,
        request: &CreateAccountRequest,
    ) -> Result<Uuid, AccountServiceError> {
        self.account_dao
            .create_account(request)
            .await
            .map_err(|a| a.into())
    }
}

#[allow(dead_code)]
pub fn get_account_service(arc: Arc<Pool>) -> Arc<dyn AccountService> {
    let dao = get_account_dao(arc);
    let service = AccountServiceImpl { account_dao: dao };
    Arc::new(service)
}

#[cfg(test)]
pub fn get_account_service_for_test(client: Arc<Pool>) -> Arc<dyn AccountService> {
    Arc::new(AccountServiceImpl {
        account_dao: get_account_dao(client),
    })
}
