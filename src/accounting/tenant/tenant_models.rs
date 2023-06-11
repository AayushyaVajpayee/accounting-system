#[derive(Debug)]
pub struct AuditMetadataBase {
    pub created_by: String,
    pub updated_by: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug)]
pub struct Tenant{
    pub id:i64,
    pub display_name:String,
    pub audit_metadata:AuditMetadataBase
}