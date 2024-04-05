use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use deadpool_postgres::Pool;
#[cfg(test)]
use mockall::automock;
use moka::future::Cache;
use thiserror::Error;
use uuid::Uuid;

use crate::common_utils::cache_utils::get_or_fetch_entity;
use crate::common_utils::dao_error::DaoError;
use crate::masters::address_master::address_service::AddressService;
use crate::masters::business_entity_master::business_entity_dao::{
    get_business_entity_dao, BusinessEntityDao,
};
use crate::masters::business_entity_master::business_entity_models::{
    BusinessEntityDto, CreateBusinessEntityRequest,
};

#[derive(Debug, Error)]
pub enum BusinessEntityServiceError {
    #[error(transparent)]
    Db(#[from] DaoError),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait BusinessEntityService: Send + Sync {
    async fn get_business_entity_by_id(
        &self,
        id: &Uuid,
        tenant_id: &Uuid,
    ) -> Result<Option<Arc<BusinessEntityDto>>, BusinessEntityServiceError>;

    async fn is_valid_business_entity_id(
        &self,
        id: &Uuid,
        tenant_id: &Uuid,
    ) -> Result<bool, BusinessEntityServiceError>;
    async fn create_business_entity(
        &self,
        request: &CreateBusinessEntityRequest,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Uuid, BusinessEntityServiceError>;
}

struct BusinessEntityServiceImpl {
    dao: Arc<dyn BusinessEntityDao>,
    address_service: Arc<dyn AddressService>,
    ///tenant_id, business_entity_id
    ///lets keep the size to be 1000 and ttl to be 5 minutes
    cache_id: Cache<(Uuid, Uuid), Arc<BusinessEntityDto>>,
}

#[allow(dead_code)]
pub fn get_business_entity_master_service(
    arc: Arc<Pool>,
    address_service: Arc<dyn AddressService>,
) -> Arc<dyn BusinessEntityService> {
    let dao = get_business_entity_dao(arc);
    let cache: Cache<(Uuid, Uuid), Arc<BusinessEntityDto>> = Cache::builder()
        .max_capacity(1000)
        .time_to_live(Duration::from_secs(300))
        .build();
    let service = BusinessEntityServiceImpl {
        dao,
        address_service,
        cache_id: cache,
    };
    Arc::new(service)
}

#[async_trait]
impl BusinessEntityService for BusinessEntityServiceImpl {
    async fn get_business_entity_by_id(
        &self,
        id: &Uuid,
        tenant_id: &Uuid,
    ) -> Result<Option<Arc<BusinessEntityDto>>, BusinessEntityServiceError> {
        get_or_fetch_entity(*tenant_id, *id, &self.cache_id, async {
            let business_entity_master = self.dao.get_business_entity(id, tenant_id).await?;
            let business_entity_dto = if let Some(entity) = business_entity_master {
                let address = if let Some(addr_id) = entity.entity_type.get_address_id() {
                    self.address_service
                        .get_address_by_id(*tenant_id, addr_id)
                        .await
                        .context("error fetching address for business entity")?
                } else {
                    None
                };
                Some(BusinessEntityDto {
                    business_entity: entity,
                    address,
                })
            } else {
                None
            };
            Ok(business_entity_dto)
        })
        .await
    }

    async fn is_valid_business_entity_id(
        &self,
        id: &Uuid,
        tenant_id: &Uuid,
    ) -> Result<bool, BusinessEntityServiceError> {
        let entity = self.get_business_entity_by_id(id, tenant_id).await?;
        return Ok(entity.is_some());
    }

    async fn create_business_entity(
        &self,
        request: &CreateBusinessEntityRequest,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Uuid, BusinessEntityServiceError> {
        let kk = self
            .dao
            .create_business_entity(request, tenant_id, user_id)
            .await?;
        Ok(kk)
    }
}
