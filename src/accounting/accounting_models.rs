
struct AuditMetadataBase{
    created_by:String,
    updated_by:String,
    created_at:u64,
    updated_at:u64
}

struct CurrencyMaster{
    id:u32,
    tenant_id:u32,
    scale:u8,
    display_name:String,
    description:String,
    audit_metadata:AuditMetadataBase
}

struct CurrencyAmount{
    scale:u8,
    amount:i64
}


struct LedgerMaster{
    id:u32,
    tenant_id:u32,
    display_name:String,
    debit_user_account_id:u128,
    credit_user_account_id:u128,
    currency_master_id:u32,
    audit_metadata:AuditMetadataBase
}

struct AccountTypeMaster{
    id:u16,
    tenant_id:u32,
    display_name:String,
    //to be used in ledger
    account_code:u8,
    audit_metadata:AuditMetadataBase
}



struct User{
    id:u128,
    tenant_id:u32,
    audit_metadata:AuditMetadataBase
}

struct UserAccount{
    id:u128,
    account_type_id:u16,
    tenant_id:u32,
    user_id:u128,
    /// should not be more than 80 char
    display_name:String,
    audit_metadata:AuditMetadataBase
}