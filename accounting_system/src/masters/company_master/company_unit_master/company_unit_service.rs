use std::sync::Arc;

use anyhow::Error as AnyhowError;
use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use thiserror::Error;
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::common_utils::pagination::pagination_utils::{PaginatedResponse, PaginationRequest};
use crate::masters::address_master::address_service::AddressService;
use crate::masters::company_master::company_unit_master::company_unit_dao::company_unit_dao::CompanyUnitDao;
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
pub trait CompanyUnitService: Send + Sync {
    async fn create_company_unit(&self, request: &CreateCompanyUnitRequest) -> Result<Uuid, CompanyUnitServiceError>;
    async fn get_company_unit_by_id(&self, id: &Uuid) -> Result<Option<CompanyUnitMaster>, CompanyUnitServiceError>;
    async fn get_company_units_by_company_id(&self, company_id: &Uuid, pagination_request: &PaginationRequest) -> Result<PaginatedResponse<CompanyUnitMaster>, CompanyUnitServiceError>;
}


struct CompanyUnitServiceImpl {
    dao: Arc<dyn CompanyUnitDao>,
    address_service: Arc<dyn AddressService>,
}

#[async_trait]
impl CompanyUnitService for CompanyUnitServiceImpl {
    async fn create_company_unit(&self, request: &CreateCompanyUnitRequest) -> Result<Uuid, CompanyUnitServiceError> {
        let jj = self.dao.create_company_unit(request).await?;
        // let k = match request.address {
        //      CompanyUnitAddressRequest::ExistingAddress { id } => {
        //          self.dao.create_company_unit_with_existing_address()
        //      }
        //      CompanyUnitAddressRequest::NewAddress { request } => {
        //
        //      }
        //  }
        Ok(jj)
    }

    async fn get_company_unit_by_id(&self, id: &Uuid) -> Result<Option<CompanyUnitMaster>, CompanyUnitServiceError> {
        todo!()
    }

    async fn get_company_units_by_company_id(&self, company_id: &Uuid, pagination_request: &PaginationRequest) -> Result<PaginatedResponse<CompanyUnitMaster>, CompanyUnitServiceError> {
        todo!()
    }
}