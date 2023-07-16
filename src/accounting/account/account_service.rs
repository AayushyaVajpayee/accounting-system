use postgres::Client;

use crate::accounting::account::account_dao::{AccountDao, get_account_dao};
use crate::accounting::account::account_models::{Account, CreateAccountRequest};

pub trait AccountService {
    fn get_account_by_id(&mut self, id: &i32) -> Option<Account>;
    fn create_account(&mut self, request: &CreateAccountRequest) -> i32;
}

struct AccountServiceImpl {
    account_dao: Box<dyn AccountDao>,
}


impl AccountService for AccountServiceImpl {
    fn get_account_by_id(&mut self, id: &i32) -> Option<Account> {
        self.account_dao.get_account_by_id(id)
    }

    fn create_account(&mut self, request: &CreateAccountRequest) -> i32 {
        self.account_dao.create_account(request)
    }
}

#[cfg(test)]
pub fn get_account_service_for_test(client: Client) -> Box<dyn AccountService> {
    let k = get_account_dao(client);
    Box::new(AccountServiceImpl { account_dao: k })
}