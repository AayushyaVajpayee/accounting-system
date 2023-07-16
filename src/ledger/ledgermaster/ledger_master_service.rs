use postgres::Client;

use crate::ledger::ledgermaster::ledger_master_dao::{get_ledger_master_dao, LedgerMasterDao};
use crate::ledger::ledgermaster::ledger_master_models::{CreateLedgerMasterEntryRequest, LedgerMaster};

pub trait LedgerMasterService {
    fn get_ledger_master_by_id(&mut self, id: &i32) -> Option<LedgerMaster>;
    fn create_ledger_master_entry(&mut self, ledger_master: &CreateLedgerMasterEntryRequest) -> i32;
}

struct LedgerMasterServiceImpl {
    ledger_master_dao: Box<dyn LedgerMasterDao>,
}

impl LedgerMasterService for LedgerMasterServiceImpl {
    fn get_ledger_master_by_id(&mut self, id: &i32) -> Option<LedgerMaster> {
        self.ledger_master_dao.get_ledger_master_by_id(id)
    }

    fn create_ledger_master_entry(&mut self, ledger_master: &CreateLedgerMasterEntryRequest) -> i32 {
        self.ledger_master_dao.create_ledger_master_entry(ledger_master)
    }
}

#[cfg(test)]
pub fn get_ledger_master_service_for_test(postgres_client: Client) -> Box<dyn LedgerMasterService> {
    let ledger_master_dao = get_ledger_master_dao(postgres_client);
    let lms = LedgerMasterServiceImpl {
        ledger_master_dao
    };
    Box::new(lms)
}