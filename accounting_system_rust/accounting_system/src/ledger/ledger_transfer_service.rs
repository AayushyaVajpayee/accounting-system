use std::sync::Arc;

use async_trait::async_trait;
use deadpool_postgres::Pool;
use uuid::Uuid;

use crate::ledger::ledger_transfer_dao::{get_ledger_transfer_dao, LedgerTransferDao};

#[async_trait]
pub trait LedgerTransferService:Send+Sync {
    async fn create_transfers(&self,request: CreateTransfersRequest);
    async fn get_transfers_by_id(&self,request: GetTransferByIdRequest);
    async fn get_transfers_for_account_for_interval(&self,request: GetTransfersForAccountForInterval);
}


struct LedgerTransferServiceImpl{
    dao:Arc<dyn LedgerTransferDao>
}

pub fn get_ledger_transfer_service(arc: Arc<Pool>) -> Arc<dyn LedgerTransferService> {
    let dao = get_ledger_transfer_dao(arc);
    let service = LedgerTransferServiceImpl{
        dao
    };
    Arc::new(service)
}
#[async_trait]
impl LedgerTransferService for LedgerTransferServiceImpl{
    async fn create_transfers(&self, _request: CreateTransfersRequest) {
        todo!()
    }

    async fn get_transfers_by_id(&self, _request: GetTransferByIdRequest) {
        todo!()
    }

    async fn get_transfers_for_account_for_interval(&self, _request: GetTransfersForAccountForInterval) {
        todo!()
    }
}


#[allow(dead_code)]
pub struct CreateTransfersRequest {
    //every inside vector fails or commits together
    //transactionally
    transfer_requests: Vec<Vec<CreateTransfersRequest>>,
}

#[allow(dead_code)]
pub struct CreateTransferRequest {
    id: Uuid,
    tenant_id: Uuid,
    caused_by_event_id: Uuid,
    grouping_id: Uuid,
    debit_account_id: i32,
    credit_account_id: i32,
    pending_id: Option<Uuid>,
    reverts_id: Option<Uuid>,
    adjusts_id: Option<Uuid>,
    timeout: Option<i64>,
    ledger_master_id: i32,
    code: i16,
    amount: i64,
    remarks: String,
    is_pending: bool,
    is_reversal: bool,
    is_adjustment: bool,
    created_at: i64,
}

#[allow(dead_code)]
pub struct CreateTransfersResponse {
    responses: Vec<CreateTransferResponse>,
}

#[allow(dead_code)]
pub struct CreateTransferResponse {
    id: Uuid,
    committed: bool,
    error_code: Option<i16>,
    error_message: Option<String>,
}

#[allow(dead_code)]
pub struct GetTransferByIdRequest {
    tenant_id: Uuid,
    id: Vec<Uuid>,
}

#[allow(dead_code)]
pub struct GetTransfersForAccountForInterval {
    tenant_id: Uuid,
    account_id: i32,
    //for now this can be 2 year
    from: i64,
    to: i64,
}

