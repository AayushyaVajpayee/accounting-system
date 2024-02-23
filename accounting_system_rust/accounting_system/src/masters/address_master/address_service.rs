use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use std::sync::Arc;
use std::time::Duration;
use deadpool_postgres::Pool;
use moka::future::Cache;
use thiserror::Error;
use uuid::Uuid;
use crate::common_utils::cache_utils::get_or_fetch_entity;

use crate::common_utils::dao_error::DaoError;
use crate::masters::address_master::address_dao::{AddressDao, get_address_dao};
use crate::masters::address_master::address_model::{Address, CreateAddressRequest};

#[derive(Debug, Error)]
pub enum AddressServiceError {
    #[error(transparent)]
    Db(#[from] DaoError)
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait AddressService: Send + Sync {
    async fn get_address_by_id(&self,tenant_id:Uuid, address_id: Uuid) -> Result<Option<Arc<Address>>, AddressServiceError>;
    async fn create_address(&self, request: &CreateAddressRequest) -> Result<Uuid, AddressServiceError>;
}


struct AddressServiceImpl {
    dao: Arc<dyn AddressDao>,
    //tenant_id, business_entity_id
    //lets keep the size to be 1000 and ttl to be 2 hour for now
    cache_id: Cache<(Uuid, Uuid), Arc<Address>>,
}

pub fn get_address_service(arc:Arc<Pool>)->Arc<dyn AddressService>{
    let dao = get_address_dao(arc);
    let cache:Cache<(Uuid,Uuid),Arc<Address>> = Cache::builder()
        .max_capacity(1000)
        .time_to_live(Duration::from_secs(2*60*60))
        .build();
    let service=AddressServiceImpl{
        dao,
        cache_id:cache
    };
    Arc::new(service)
}
#[async_trait]
impl AddressService for AddressServiceImpl {
    async fn get_address_by_id(&self,tenant_id:Uuid, address_id: Uuid)
        -> Result<Option<Arc<Address>>, AddressServiceError> {
        get_or_fetch_entity(tenant_id,address_id,
        &self.cache_id,
            async{
                let p = self.dao.get_address_by_id(tenant_id,address_id).await?;
                Ok(p)
            }
        ).await
    }

    async fn create_address(&self, request: &CreateAddressRequest) -> Result<Uuid, AddressServiceError> {
        let a = self.dao.create_address(request).await?;
        Ok(a)
    }
}