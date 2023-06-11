#[derive(Debug)]
pub struct AuditMetadataBase {
    pub created_by: String,
    pub updated_by: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug)]
pub struct CurrencyMaster {
    pub id: i32,
    pub tenant_id: i32,
    pub scale: i16,
    ///16 char
    pub display_name: String,
    ///50 char
    pub description: String,
    pub audit_metadata: AuditMetadataBase,
}

struct CurrencyAmount {
    scale: i16,
    amount: i64,
}
