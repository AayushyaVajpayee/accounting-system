use std::sync::Arc;
use async_trait::async_trait;
use crate::accounting::postgres_factory::get_postgres_conn_pool;

use crate::ledger::ledgermaster::ledger_master_dao::{get_ledger_master_dao, LedgerMasterDao};
use crate::ledger::ledgermaster::ledger_master_models::{CreateLedgerMasterEntryRequest, LedgerMaster};

#[async_trait]
pub trait LedgerMasterService:Send+Sync {
    async fn get_ledger_master_by_id(&self, id: &i32) -> Option<LedgerMaster>;
    async fn create_ledger_master_entry(&self, ledger_master: &CreateLedgerMasterEntryRequest) -> i32;
}

struct LedgerMasterServiceImpl {
    dao: Arc<dyn LedgerMasterDao>,
}

#[async_trait]
impl LedgerMasterService for LedgerMasterServiceImpl {
    async fn get_ledger_master_by_id(&self, id: &i32) -> Option<LedgerMaster> {
        self.dao.get_ledger_master_by_id(id).await
    }

    async fn create_ledger_master_entry(&self, ledger_master: &CreateLedgerMasterEntryRequest) -> i32 {
        self.dao.create_ledger_master_entry(ledger_master).await
    }
}


pub fn get_ledger_master_service()->Arc<dyn LedgerMasterService>{
    let pclient = get_postgres_conn_pool();
    let dao = get_ledger_master_dao(pclient);
    let service = LedgerMasterServiceImpl{
        dao
    };
    Arc::new(service)
}

#[cfg(test)]
pub fn get_ledger_master_service_for_test(postgres_client: &'static deadpool_postgres::Pool) -> Arc<dyn LedgerMasterService> {
    let ledger_master_dao =get_ledger_master_dao(postgres_client);
    Arc::new(LedgerMasterServiceImpl {
        dao: ledger_master_dao
    })
}