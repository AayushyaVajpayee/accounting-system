use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use uuid::Uuid;
use crate::ledger::ledger_models::TransferType::Regular;

#[derive(Debug, Deserialize)]
pub struct TransferCreationDbResponse {
    pub txn_id: Uuid,
    pub committed: bool,
    pub reason: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransferType {
    Regular,
    Pending,
    PostPending { pending_id: Uuid },
    //accounts, ledger_id, don't make much sense here. I need to wrtie test if i include them otherwise i dont
    VoidPending { pending_id: Uuid },//accounts, ledger_id,amount don't make much sense here. I need to wrtie test if i include them otherwise i dont
}

//todo create a validation function for transfer
//todo it should validate that correct parameters are set and incorrect combination of parameters cannot be set
#[derive(Debug, Clone)]
pub struct Transfer {
    pub id: Uuid,
    pub tenant_id: i32,
    pub debit_account_id: i32,
    pub credit_account_id: i32,
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
    pub transfer_type: TransferType,
    pub created_at: i64,
}



#[derive(Default)]
pub struct TransferBuilder {
    pub id: Option<Uuid>,
    pub tenant_id: Option<i32>,
    pub debit_account_id: Option<i32>,
    pub credit_account_id: Option<i32>,
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
    pub transfer_type: Option<TransferType>
}

#[cfg(test)]
pub fn a_transfer(builder: TransferBuilder) -> Transfer {
    Transfer {
        id: builder.id.unwrap_or_else(Uuid::new_v4),
        tenant_id: builder.tenant_id.unwrap_or(1),
        debit_account_id: builder.debit_account_id.unwrap_or(0),
        credit_account_id: builder.credit_account_id.unwrap_or(1),
        caused_by_event_id: builder.caused_by_event_id.unwrap_or_else(Uuid::new_v4),
        grouping_id: builder.grouping_id.unwrap_or_else(Uuid::new_v4),
        ledger_master_id: builder.ledger_master_id.unwrap_or(1),
        code: builder.code.unwrap_or(0),
        amount: builder.amount.unwrap_or(100),
        remarks: builder.remarks,
        created_at: builder.created_at.unwrap_or_else(|| SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as i64),
        transfer_type: builder.transfer_type.unwrap_or(Regular),
    }
}