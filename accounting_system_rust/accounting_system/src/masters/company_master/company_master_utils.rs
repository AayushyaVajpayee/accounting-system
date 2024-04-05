use anyhow::anyhow;
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::common_utils::utils::get_current_time_us;
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;
use crate::masters::company_master::company_master_models::company_identification_number::CompanyIdentificationNumber;
use crate::masters::company_master::company_master_models::company_master::CompanyMaster;
use crate::masters::company_master::company_master_models::company_name::CompanyName;
use crate::masters::company_master::company_master_models::master_status_enum::MasterStatusEnum;
use crate::masters::company_master::company_master_request_response::CreateCompanyRequest;

impl CreateCompanyRequest {
    pub fn to_company_master(&self) -> anyhow::Result<CompanyMaster> {
        Ok(CompanyMaster {
            base_master_fields: BaseMasterFields {
                id: Uuid::now_v7(),
                entity_version_id: 0,
                tenant_id: self.tenant_id,
                active: false,
                approval_status: MasterStatusEnum::Approved,
                remarks: None,
            },
            name: CompanyName::new(self.name.as_str()).map_err(|a| anyhow!(a))?,
            cin: CompanyIdentificationNumber::new(self.cin.as_str()).map_err(|a| anyhow!(a))?,
            audit_metadata: AuditMetadataBase {
                created_by: self.created_by,
                updated_by: self.created_by,
                created_at: get_current_time_us()?,
                updated_at: get_current_time_us()?,
            },
        })
    }
}
