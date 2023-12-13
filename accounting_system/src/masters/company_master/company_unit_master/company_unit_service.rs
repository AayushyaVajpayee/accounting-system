use anyhow::Error as AnyhowError;
use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use thiserror::Error;
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::common_utils::pagination::pagination_utils::{PaginatedResponse, PaginationRequest};
use crate::masters::company_master::company_unit_master::company_unit_models::{CompanyUnitMaster, CreateCompanyUnitRequest};

#[derive(Debug, Error)]
pub enum CompanyUnitServiceError {
    #[error(transparent)]
    Db(#[from] DaoError),
    #[error(transparent)]
    Other(#[from] AnyhowError),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CompanyUnitService {
    async fn create_company_unit(&self, request: &CreateCompanyUnitRequest) -> Result<Uuid, CompanyUnitServiceError>;
    async fn get_company_unit_by_id(&self, id: &Uuid) -> Result<Option<CompanyUnitMaster>, CompanyUnitServiceError>;
    async fn get_company_units_by_company_id(&self, company_id: &Uuid, pagination_request: &PaginationRequest) -> Result<PaginatedResponse<CompanyUnitMaster>, CompanyUnitServiceError>;
}