use uuid::Uuid;

pub trait LedgerTransferService {
    fn create_transfers(request: CreateTransfersRequest);
    fn get_transfers_by_id(request: GetTransferByIdRequest);
    fn get_transfers_for_account_for_interval(request: GetTransfersForAccountForInterval);
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
    tenant_id: i32,
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
    tenant_id: i32,
    id: Vec<Uuid>,
}

#[allow(dead_code)]
pub struct GetTransfersForAccountForInterval {
    tenant_id: i32,
    account_id: i32,
    //for now this can be 2 year
    from: i64,
    to: i64,
}

