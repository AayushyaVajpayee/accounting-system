use async_trait::async_trait;
use deadpool_postgres::Pool;
#[cfg(test)]
use mockall::automock;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use crate::accounting::currency::currency_dao::{CurrencyDao, get_currency_dao};
use crate::accounting::currency::currency_models::{CreateCurrencyMasterRequest, CurrencyMaster};
use crate::common_utils::dao_error::DaoError;

#[derive(Debug, Error)]
pub enum CurrencyServiceError {
    #[error(transparent)]
    Db(#[from] DaoError)
}


#[cfg_attr(test, automock)]
#[async_trait]
pub trait CurrencyService: Send + Sync {
    async fn create_currency_entry(&self, request: &CreateCurrencyMasterRequest,tenant_id:Uuid) -> Result<Uuid, CurrencyServiceError>;
    async fn get_currency_entry(&self, id: Uuid,tenant_id:Uuid) -> Result<Option<CurrencyMaster>, CurrencyServiceError>;
}

struct CurrencyServiceImpl {
    currency_dao: Arc<dyn CurrencyDao>,
}

#[allow(dead_code)]
pub fn get_currency_service(arc: Arc<Pool>) -> Arc<dyn CurrencyService> {
    let currency_dao = get_currency_dao(arc);
    let currency_s = CurrencyServiceImpl { currency_dao };
    Arc::new(currency_s)
}

#[allow(dead_code)]
#[cfg(test)]
pub fn get_currency_service_for_test(postgres_client: Arc<Pool>) -> Arc<dyn CurrencyService> {
    let currency_dao = get_currency_dao(postgres_client);
    let currency_service = CurrencyServiceImpl {
        currency_dao
    };
    Arc::new(currency_service)
}

#[async_trait]
impl CurrencyService for CurrencyServiceImpl {
    async fn create_currency_entry(&self, request: &CreateCurrencyMasterRequest,tenant_id:Uuid) -> Result<Uuid, CurrencyServiceError> {
        self.currency_dao.create_currency_entry(request,tenant_id).await.map_err(|a| a.into())
    }
    async fn get_currency_entry(&self, id:Uuid,tenant_id:Uuid) -> Result<Option<CurrencyMaster>, CurrencyServiceError> {
        self.currency_dao.get_currency_entry_by_id(id,tenant_id).await.map_err(|a| a.into())
    }
}