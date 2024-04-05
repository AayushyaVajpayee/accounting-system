use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use async_trait::async_trait;
use deadpool_postgres::Pool;
#[cfg(test)]
use mockall::automock;
use moka::future::Cache;
use thiserror::Error;
use uuid::Uuid;

use crate::common_utils::cache_utils::get_or_fetch_entity;
use crate::common_utils::dao_error::DaoError;
use crate::masters::address_master::address_dao::{AddressDao, get_address_dao};
use crate::masters::address_master::address_model::{AddressDto, CreateAddressRequest};
use crate::masters::city_master::city_master_service::CityMasterService;
use crate::masters::country_master::country_service::CountryMasterService;
use crate::masters::pincode_master::pincode_master_service::PincodeMasterService;
use crate::masters::state_master::state_master_service::StateMasterService;

#[derive(Debug, Error)]
pub enum AddressServiceError {
    #[error(transparent)]
    Db(#[from] DaoError),
    #[error(transparent)]
    Other(#[from] anyhow::Error)
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait AddressService: Send + Sync {
    async fn get_address_by_id(&self,tenant_id:Uuid, address_id: Uuid) -> Result<Option<Arc<AddressDto>>, AddressServiceError>;
    async fn create_address(&self, request: &CreateAddressRequest,tenant_id:Uuid,user_id:Uuid) -> Result<Uuid, AddressServiceError>;
}


struct AddressServiceImpl {
    dao: Arc<dyn AddressDao>,
    country_service:Arc<dyn CountryMasterService>,
    city_service:Arc<dyn CityMasterService>,
    pincode_service:Arc<dyn PincodeMasterService>,
    state_service:Arc<dyn StateMasterService>,
    //tenant_id, business_entity_id
    //lets keep the size to be 1000 and ttl to be 2 hour for now
    cache_id: Cache<(Uuid, Uuid), Arc<AddressDto>>,
}

pub fn get_address_service(arc:Arc<Pool>,
                           country_service:Arc<dyn CountryMasterService>,
                           city_service:Arc<dyn CityMasterService>,
                           pincode_service:Arc<dyn PincodeMasterService>,
                           state_service:Arc<dyn StateMasterService> )->Arc<dyn AddressService>{
    let dao = get_address_dao(arc);
    let cache:Cache<(Uuid,Uuid),Arc<AddressDto>> = Cache::builder()
        .max_capacity(1000)
        .time_to_live(Duration::from_secs(2*60*60))
        .build();
    let service=AddressServiceImpl{
        dao,
        country_service,
        city_service,
        state_service,
        pincode_service,
        cache_id:cache
    };
    Arc::new(service)
}
#[async_trait]
impl AddressService for AddressServiceImpl {
    async fn get_address_by_id(&self,tenant_id:Uuid, address_id: Uuid)
        -> Result<Option<Arc<AddressDto>>, AddressServiceError> {
        get_or_fetch_entity(tenant_id,address_id,
        &self.cache_id,
            async{
                let p = self.dao.get_address_by_id(tenant_id,address_id).await?;
                if let Some(add)=p{
                    let pincode =self.pincode_service.get_pincode_by_id(&add.pincode_id).await
                        .ok_or_else(||anyhow!("pincode not found for pincode_id in address"))?;
                    let state = self.state_service.get_state_by_id(&add.state_id).await
                        .ok_or_else(||anyhow!("state not found for state_id in address"))?;
                    let country = self.country_service.get_country_by_id(add.country_id).await
                        .ok_or_else(||anyhow!("country not found for country_id in address"))?;
                    let city = self.city_service.get_city_by_id(&add.city_id).await
                        .ok_or_else(||anyhow!("city not found for city_id in address"))?;
                    let dto = AddressDto{
                        address: add,
                        country,
                        city,
                        pincode,
                        state,
                    };
                    Ok(Some(dto))
                }else{
                    Ok(None)
                }
            }
        ).await
    }

    async fn create_address(&self, request: &CreateAddressRequest,tenant_id:Uuid,user_id:Uuid) -> Result<Uuid, AddressServiceError> {
        let a = self.dao.create_address(request,tenant_id,user_id).await?;
        Ok(a)
    }
}