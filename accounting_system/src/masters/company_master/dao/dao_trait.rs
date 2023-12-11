use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use uuid::Uuid;
use crate::common_utils::dao_error::DaoError;
use crate::common_utils::pagination::pagination_utils::PaginatedResponse;
use crate::masters::company_master::company_master_models::company_master::CompanyMaster;
use crate::masters::company_master::company_master_models::master_updation_remarks::MasterUpdationRemarks;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CompanyMasterDao: Send + Sync {
    async fn get_company_by_id(
        &self,
        tenant_id: Uuid,
        company_id: Uuid,
    ) -> Result<Option<CompanyMaster>, DaoError>;
    async fn get_all_companies_for_tenant(
        &self,
        tenant_id: &Uuid,
        page_no: u32,
        per_page: u32,
    ) -> Result<PaginatedResponse<CompanyMaster>, DaoError>;
    async fn create_new_company_for_tenant(&self, entity: &CompanyMaster, idempotence_key: &Uuid) -> Result<Uuid, DaoError>;
    // async fn update_company_data_for_tenant(&self);
    async fn soft_delete_company_for_tenant(
        &self,
        tenant_id: Uuid,
        company_id: Uuid,
        entity_version_id: i32,
        remarks: &MasterUpdationRemarks,
        updated_by: Uuid,
    ) -> Result<u64, DaoError>;
}
