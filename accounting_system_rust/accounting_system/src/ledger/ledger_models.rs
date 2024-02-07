use serde::Deserialize;
use uuid::Uuid;


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

#[derive(Debug, Clone)]
pub struct Transfer {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub debit_account_id: Uuid,
    pub credit_account_id: Uuid,
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
    pub ledger_master_id: Uuid,
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
    pub tenant_id: Option<Uuid>,
    pub debit_account_id: Option<Uuid>,
    pub credit_account_id: Option<Uuid>,
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
    pub ledger_master_id: Option<Uuid>,
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
pub mod tests {
    use uuid::Uuid;

    use crate::accounting::account::account_models::tests::{SEED_CREDIT_ACCOUNT_ID, SEED_DEBIT_ACCOUNT_ID};
    use crate::ledger::ledger_models::{Transfer, TransferBuilder};
    use crate::ledger::ledgermaster::ledger_master_models::tests::SEED_LEDGER_MASTER_ID;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    pub fn a_transfer(builder: TransferBuilder) -> Transfer {
        Transfer {
            id: builder.id.unwrap_or_else(Uuid::now_v7),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            debit_account_id: builder.debit_account_id.unwrap_or(*SEED_DEBIT_ACCOUNT_ID),
            credit_account_id: builder.credit_account_id.unwrap_or(*SEED_CREDIT_ACCOUNT_ID),
            caused_by_event_id: builder.caused_by_event_id.unwrap_or_else(Uuid::now_v7),
            grouping_id: builder.grouping_id.unwrap_or_else(Uuid::now_v7),
            ledger_master_id: builder.ledger_master_id.unwrap_or(*SEED_LEDGER_MASTER_ID),
            code: builder.code.unwrap_or(0),
            amount: builder.amount.unwrap_or(100),
            remarks: builder.remarks,
            created_at: builder.created_at.unwrap_or_else(|| std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() as i64),
            transfer_type: builder.transfer_type.unwrap_or(crate::ledger::ledger_models::TransferType::Regular),
        }
    }
}
