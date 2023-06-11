use crate::accounting::currency::currency_models::AuditMetadataBase;

#[derive(Debug)]
pub struct Tenant{
    pub id:i64,
    pub display_name:String,
    pub audit_metadata:AuditMetadataBase
}