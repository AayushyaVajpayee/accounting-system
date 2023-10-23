use anyhow::anyhow;
use uuid::Uuid;
use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::common_utils::utils::get_current_time_us;
use crate::masters::company_master::company_master_model::{BaseMasterFields, CompanyIdentificationNumber, CompanyMaster, CompanyName, MasterStatusEnum};
use crate::masters::company_master::company_master_requests::CreateCompanyRequest;

impl CreateCompanyRequest{
    pub fn to_company_master(&self)->anyhow::Result<CompanyMaster>{
        Ok(CompanyMaster{
            base_master_fields: BaseMasterFields {
                id:Uuid::now_v7(),
                entity_version_id: 0,
                tenant_id: self.tenant_id,
                active: false,
                approval_status: MasterStatusEnum::Approved,
                remarks: None,
            },
            name: CompanyName::new(self.name.as_str())
                .map_err(|a|anyhow!(a))?,
            cin: CompanyIdentificationNumber::new(self.cin.as_str())
                .map_err(|a|anyhow!(a))?,
            audit_metadata: AuditMetadataBase{
                created_by: self.created_by,
                updated_by: self.created_by,
                created_at: get_current_time_us()?,
                updated_at: get_current_time_us()?,
            },
        })
    }
}