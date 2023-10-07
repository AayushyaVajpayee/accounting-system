use uuid::Uuid;
use crate::accounting::currency::currency_models::AuditMetadataBase;

pub struct CompanyMaster{
    id:Uuid,
    name:String,//50  chars maximum
    gstin:String,
    audit_metadata:AuditMetadataBase

}

