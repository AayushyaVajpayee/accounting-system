use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::common_utils::pagination::pagination_utils::PaginatedResponse;
use crate::masters::company_master::company_unit_master::company_unit_models::{
    CompanyUnitMaster, CreateCompanyUnitRequest,
};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CompanyUnitDao: Send + Sync {
    async fn create_company_unit(
        &self,
        request: &CreateCompanyUnitRequest,tenant_id:Uuid,user_id:Uuid
    ) -> Result<Uuid, DaoError>;
    async fn get_company_unit_by_id(
        &self,
        id: &Uuid,
    ) -> Result<Option<CompanyUnitMaster>, DaoError>;
    async fn get_company_units_by_company_id(
        &self,
        company_id: &Uuid,
        page_no: u32,
        per_page: u32,
    ) -> Result<PaginatedResponse<CompanyUnitMaster>, DaoError>;
}
