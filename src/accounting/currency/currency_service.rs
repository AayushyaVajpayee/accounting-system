use async_trait::async_trait;
use deadpool_postgres::Pool;
use crate::accounting::currency::currency_dao::{CurrencyDao, get_currency_dao};
use crate::accounting::currency::currency_models::{CreateCurrencyMasterRequest, CurrencyMaster};
use crate::accounting::postgres_factory::get_postgres_conn_pool;

#[async_trait]
pub trait CurrencyService {
    async fn create_currency_entry(&self, request: &CreateCurrencyMasterRequest) -> i16;
    async fn get_currency_entry(&self, id: &i16) -> Option<CurrencyMaster>;
}

struct CurrencyServiceImpl {
    currency_dao: Box<dyn CurrencyDao + Send + Sync>,
}

#[allow(dead_code)]
pub async fn get_currency_service() -> Box<dyn CurrencyService + Send + Sync> {
    let pclient = get_postgres_conn_pool().await;
    let currency_dao = get_currency_dao(pclient);
    let currency_s = CurrencyServiceImpl { currency_dao };
    Box::new(currency_s)
}

#[allow(dead_code)]
#[cfg(test)]
pub fn get_currency_service_for_test(postgres_client: &'static Pool) -> Box<dyn CurrencyService + Send + Sync> {
    let currency_dao = get_currency_dao(postgres_client);
    let currency_service = CurrencyServiceImpl {
        currency_dao
    };
    Box::new(currency_service)
}

#[async_trait]
impl CurrencyService for CurrencyServiceImpl {
    async fn create_currency_entry(&self, request: &CreateCurrencyMasterRequest) -> i16 {
        self.currency_dao.create_currency_entry(request).await
    }
    async fn get_currency_entry(&self, id: &i16) -> Option<CurrencyMaster> {
        self.currency_dao.get_currency_entry_by_id(id).await
    }
}