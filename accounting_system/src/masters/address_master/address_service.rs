use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use thiserror::Error;
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::masters::address_master::address_dao::AddressDao;
use crate::masters::address_master::address_model::{Address, CreateAddressRequest};

#[derive(Debug, Error)]
pub enum AddressServiceError {
    #[error(transparent)]
    Db(#[from] DaoError)
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait AddressService: Send + Sync {
    async fn get_address_by_id(&self, address_id: &Uuid) -> Result<Option<Address>, AddressServiceError>;
    async fn create_address(&self, request: &CreateAddressRequest) -> Result<Uuid, AddressServiceError>;
}


struct AddressServiceImpl {
    dao: Arc<dyn AddressDao>,
}

#[async_trait]
impl AddressService for AddressServiceImpl {
    async fn get_address_by_id(&self, address_id: &Uuid) -> Result<Option<Address>, AddressServiceError> {
        let addr = self.dao.get_address_by_id(address_id).await?;
        Ok(addr)
    }

    async fn create_address(&self, request: &CreateAddressRequest) -> Result<Uuid, AddressServiceError> {
        let a = self.dao.create_address(request).await?;
        Ok(a)
    }
}