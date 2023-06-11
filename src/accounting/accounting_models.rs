
struct AuditMetadataBase{
    created_by:String,
    updated_by:String,
    created_at:i64,
    updated_at:i64
}


struct LedgerMaster{
    id:i32,
    tenant_id:i32,
    display_name:String,
    debit_user_account_id:i64,
    credit_user_account_id:i64,
    currency_master_id:i32,
    audit_metadata:AuditMetadataBase
}

struct AccountTypeMaster{
    id:i16,
    tenant_id:i32,
    display_name:String,
    //to be used in ledger
    account_code:i8,
    audit_metadata:AuditMetadataBase
}



struct User{
    id:i64,
    tenant_id:i32,
    audit_metadata:AuditMetadataBase
}

struct UserAccount{
    id:i128,
    account_type_id:i16,
    tenant_id:i32,
    user_id:i128,
    /// should not be more than 80 char
    display_name:String,
    audit_metadata:AuditMetadataBase
}