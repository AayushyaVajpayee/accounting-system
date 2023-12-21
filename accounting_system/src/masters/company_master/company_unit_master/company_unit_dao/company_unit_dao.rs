use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::masters::company_master::company_unit_master::company_unit_models::{CompanyUnitMaster, CreateCompanyUnitRequest};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CompanyUnitDao: Send + Sync {
    async fn create_company_unit(&self, request: &CreateCompanyUnitRequest) -> Result<Uuid, DaoError>;
    async fn get_company_unit_by_id(&self, id: &Uuid) -> Result<Option<CompanyUnitMaster>, DaoError>;
    async fn get_company_units_by_company_id(&self, company_id: &Uuid) -> Result<Vec<CompanyUnitMaster>, DaoError>;
}

