use uuid::Uuid;
use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;
use crate::masters::company_master::company_master_models::gstin_no::GstinNo;

#[derive(Debug)]
pub struct CompanyUnitMaster {
    base_master_fields: BaseMasterFields,
    company_id: Uuid,
    address_id: Uuid,
    gstin: GstinNo,
    audit_metadata: AuditMetadataBase,
}