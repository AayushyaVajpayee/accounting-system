use postgres::Client;

use crate::accounting::currency::currency_dao::{CurrencyDao, get_currency_dao};
use crate::accounting::currency::currency_models::{CreateCurrencyMasterRequest, CurrencyMaster};
use crate::accounting::postgres_factory::create_postgres_client;

pub trait CurrencyService {
    fn create_currency_entry(&mut self, request: &CreateCurrencyMasterRequest) -> i16;
    fn get_currency_entry(&mut self, id: &i16) -> Option<CurrencyMaster>;
}

struct CurrencyServiceImpl {
    currency_dao: Box<dyn CurrencyDao>,
}

#[allow(dead_code)]
pub fn get_currency_service() -> Box<dyn CurrencyService> {
    let pclient = create_postgres_client();
    let currency_dao = get_currency_dao(pclient);
    let currency_s = CurrencyServiceImpl { currency_dao: currency_dao };
    Box::new(currency_s)
}

#[allow(dead_code)]
#[cfg(test)]
pub fn get_currency_service_for_test(postgres_client: Client) -> Box<dyn CurrencyService> {
    let currency_dao = get_currency_dao(postgres_client);
    let currency_service = CurrencyServiceImpl {
        currency_dao: currency_dao
    };
    Box::new(currency_service)
}

impl CurrencyService for CurrencyServiceImpl {
    fn create_currency_entry(&mut self, request: &CreateCurrencyMasterRequest) -> i16 {
        self.currency_dao.create_currency_entry(request)
    }
    fn get_currency_entry(&mut self, id: &i16) -> Option<CurrencyMaster> {
        self.currency_dao.get_currency_entry_by_id(id)
    }
}