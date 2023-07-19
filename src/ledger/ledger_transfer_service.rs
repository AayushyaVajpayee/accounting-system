use std::ops::Not;
use uuid::Uuid;
use crate::ledger::ledger_models::Transfer;

pub trait LedgerTransferService {
    fn create_transfers(request: CreateTransfersRequest);
    fn get_transfers_by_id(request: GetTransferByIdRequest);
    fn get_transfers_for_account_for_interval(request: GetTransfersForAccountForInterval);
}

pub fn validate_transfer_object(t: &Transfer) -> Vec<String> {
    let mut errors: Vec<String> = vec![];
    errors
}

pub struct CreateTransfersRequest {
    //every inside vector fails or commits together
    //transactionally
    transfer_requests: Vec<Vec<CreateTransfersRequest>>,
}

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

pub struct CreateTransfersResponse {
    responses: Vec<CreateTransferResponse>,
}

pub struct CreateTransferResponse {
    id: Uuid,
    committed: bool,
    error_code: Option<i16>,
    error_message: Option<String>,
}

pub struct GetTransferByIdRequest {
    tenant_id: i32,
    id: Vec<Uuid>,
}

pub struct GetTransfersForAccountForInterval {
    tenant_id: i32,
    account_id: i32,
    //for now this can be 2 year
    from: i64,
    to: i64,
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use uuid::Uuid;
    use crate::ledger::ledger_models::{a_transfer, TransferBuilder};
    use crate::ledger::ledger_transfer_service::validate_transfer_object;

    #[test]
    fn should_not_return_error_message_for_correct_transfer() {
        let transfer = a_transfer(
            Default::default());
        let res = validate_transfer_object(&transfer);
        assert_eq!(0, res.len());
    }
}