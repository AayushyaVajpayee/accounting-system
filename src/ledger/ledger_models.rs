use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct TransferCreationDbResponse {
    pub txn_id: Uuid,
    pub committed: bool,
    pub reason: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Transfer {
    pub id: Uuid,
    pub tenant_id: i32,
    pub debit_account_id: i32,
    pub credit_account_id: i32,
    pub pending_id: Option<Uuid>,
    pub reverts_id: Option<Uuid>,
    pub adjusts_id: Option<Uuid>,
    // this will be physical event like an invoice
    // generating 3-4 entries:tax ledger entry, tds entry, taxable entry, and invoice payable entry
    pub caused_by_event_id: Uuid,
    // for an order id,
    // invoice gets generated, 4 entries get create
    // invoice gets reverted
    // credit note get generated
    // debit notes get generated
    // customer payment received at a later date
    // all these will be have same grouping_id
    pub grouping_id: Uuid,
    pub timeout: Option<i64>,
    //this is basically partitioning the set of accounts that can transact together,
    //one reason can be this can have the same currency
    //another is ease of doing database partitioning
    pub ledger_master_id: i32,
    ///this code is actually a reference to a transaction type. we will need to maintain that master too
    pub code: i16,
    //will need to check this is always positive
    pub amount: i64,
    /// should be max 80 char
    pub remarks: Option<String>,
    pub created_at: i64,
    pub is_pending: bool,
    pub post_pending: bool,
    pub void_pending: bool,
    pub is_reversal: bool,
    pub is_adjustment: bool,
}

#[derive(Default)]
pub struct TransferBuilder {
    pub id: Option<Uuid>,
    pub tenant_id: Option<i32>,
    pub debit_account_id: Option<i32>,
    pub credit_account_id: Option<i32>,
    pub pending_id: Option<Uuid>,
    pub reverts_id: Option<Uuid>,
    pub adjusts_id: Option<Uuid>,
    // this will be physical event like an invoice
    // generating 3-4 entries:tax ledger entry, tds entry, taxable entry, and invoice payable entry
    pub caused_by_event_id: Option<Uuid>,
    // for an order id,
    // invoice gets generated, 4 entries get create
    // invoice gets reverted
    // credit note get generated
    // debit notes get generated
    // customer payment received at a later date
    // all these will be have same grouping_id
    pub grouping_id: Option<Uuid>,
    pub timeout: Option<i64>,
    //this is basically partitioning the set of accounts that can transact together,
    //one reason can be this can have the same currency
    pub ledger_master_id: Option<i32>,
    ///this code is actually a reference to a transaction type. we will need to maintain that master too
    pub code: Option<i16>,
    //will need to check this is always positive
    pub amount: Option<i64>,
    /// should be max 80 char
    pub remarks: Option<String>,
    pub created_at: Option<i64>,
    pub is_pending: Option<bool>,
    pub post_pending: Option<bool>,
    pub void_pending: Option<bool>,
    pub is_reversal: Option<bool>,
    pub is_adjustment: Option<bool>,
}

#[cfg(test)]
pub fn a_transfer(builder: TransferBuilder) -> Transfer {
    Transfer {
        id: builder.id.unwrap_or_else(Uuid::new_v4),
        tenant_id: builder.tenant_id.unwrap_or(1),
        debit_account_id: builder.debit_account_id.unwrap_or(0),
        credit_account_id: builder.credit_account_id.unwrap_or(1),
        pending_id: builder.pending_id,
        reverts_id: builder.reverts_id,
        adjusts_id: builder.adjusts_id,
        caused_by_event_id: builder.caused_by_event_id.unwrap_or_else(Uuid::new_v4),
        grouping_id: builder.grouping_id.unwrap_or_else(Uuid::new_v4),
        timeout: builder.timeout,
        ledger_master_id: builder.ledger_master_id.unwrap_or(0),
        code: builder.code.unwrap_or(0),
        amount: builder.amount.unwrap_or(100),
        remarks: builder.remarks,
        created_at: builder.created_at.unwrap_or_else(|| SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as i64),
        is_pending: builder.is_pending.unwrap_or(false),
        post_pending: builder.post_pending.unwrap_or(false),
        void_pending: builder.void_pending.unwrap_or(false),
        is_reversal: builder.is_reversal.unwrap_or(false),
        is_adjustment: builder.is_adjustment.unwrap_or(false),
    }
}