use uuid::Uuid;

struct Transfer {
    id: Uuid,
    debit_account_id: i32,
    credit_account_id: i32,
    pending_id: Option<Uuid>,
    reverts_id: Option<Uuid>,
    adjusts_id: Option<Uuid>,
    // this will be physical event like an invoice
    // generating 3-4 entries:tax ledger entry, tds entry, taxable entry, and invoice payable entry
    caused_by_event_id: Uuid,
    // for an order id,
    // invoice gets generated, 4 entries get create
    // invoice gets reverted
    // credit note get generated
    // debit notes get generated
    // customer payment received at a later date
    // all these will be have same grouping_id
    grouping_id: Uuid,
    timeout: Option<i64>,
    //this is basically partitioning the set of accounts that can transact together,
    //one reason can be this can have the same currency
    ledger_master_id: i32,
    code: i16,
    //will need to check this is always positive
    amount: i64,
    /// should be max 80 char
    remarks: Option<String>,
    created_at: i64,
    is_pending: bool,
    post_pending: bool,
    void_pending: bool,
    is_reversal: bool,
    is_adjustment: bool,
}
