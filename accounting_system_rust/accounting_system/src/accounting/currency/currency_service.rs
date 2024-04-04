use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use deadpool_postgres::Pool;
#[cfg(test)]
use mockall::automock;
use moka::future::Cache;
use thiserror::Error;
use uuid::Uuid;

use crate::accounting::currency::currency_dao::{CurrencyDao, get_currency_dao};
use crate::accounting::currency::currency_models::{CreateCurrencyMasterRequest, CurrencyMaster};
use crate::common_utils::cache_utils::get_or_fetch_entity;
use crate::common_utils::dao_error::DaoError;

#[derive(Debug, Error)]
pub enum CurrencyServiceError {
    #[error(transparent)]
    Db(#[from] DaoError)
}


#[cfg_attr(test, automock)]
#[async_trait]
pub trait CurrencyService: Send + Sync {
    async fn create_currency_entry(&self, request: &CreateCurrencyMasterRequest, tenant_id: Uuid,user_id:Uuid) -> Result<Uuid, CurrencyServiceError>;
    async fn get_currency_entry(&self, id: Uuid, tenant_id: Uuid) -> Result<Option<Arc<CurrencyMaster>>, CurrencyServiceError>;
}

struct CurrencyServiceImpl {
    currency_dao: Arc<dyn CurrencyDao>,
    cache_by_tenant_id_and_currency_id: Cache<(Uuid, Uuid), Arc<CurrencyMaster>>,
}

pub fn get_currency_service(arc: Arc<Pool>) -> Arc<dyn CurrencyService> {
    let currency_dao = get_currency_dao(arc);
    let cache: Cache<(Uuid, Uuid), Arc<CurrencyMaster>> = Cache::builder()
        .max_capacity(1000)
        .time_to_live(Duration::from_secs(300))
        .build();
    let currency_s = CurrencyServiceImpl { currency_dao, cache_by_tenant_id_and_currency_id: cache };
    Arc::new(currency_s)
}

#[async_trait]
impl CurrencyService for CurrencyServiceImpl {
    async fn create_currency_entry(&self, request: &CreateCurrencyMasterRequest, tenant_id: Uuid,user_id:Uuid) -> Result<Uuid, CurrencyServiceError> {
        self.currency_dao.create_currency_entry(request, tenant_id,user_id).await.map_err(|a| a.into())
    }
    async fn get_currency_entry(&self, id: Uuid, tenant_id: Uuid) -> Result<Option<Arc<CurrencyMaster>>, CurrencyServiceError> {
        let fetch = async
            {
                let p = self.currency_dao
                    .get_currency_entry_by_id(id, tenant_id).await?;
                Ok(p)
            };
        get_or_fetch_entity(tenant_id, id, &self.cache_by_tenant_id_and_currency_id,
                            fetch).await
    }
}
