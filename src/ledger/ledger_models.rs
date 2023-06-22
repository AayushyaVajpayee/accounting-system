struct Transfer{
    id:u128,
    debit_account_id:u128,
    credit_account_id:u128,
    pending_id:u128,
    timeout:u64,
    ledger:u32,
    code:u16,
    amount:u64,
    /// should be max 80 char
    remarks:String,
    timestamp:u64,
    flags:TransferFlags,

}

struct TransferFlags{
    linked:bool,
    pending:bool,
    post_pending_transfer:bool,
    void_pending_transfer:bool,
    balancing_debit:bool,
    balancing_credit:bool,
}

struct Account{
    id:u128,
    /// ledger can contain the currency
    ledger:u32,
    code:u16,
    debits_pending:u64,
    debits_posted:u64,
    credits_pending:u64,
    credits_posted:u64,
    timestamp:u64,
    flags:AccountFlags
}


struct AccountFlags{
    linked:bool,
    debits_must_not_exceed_credits:bool,
    credits_must_not_exceed_debits:bool

}