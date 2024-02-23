use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use deadpool_postgres::Pool;
#[cfg(test)]
use mockall::automock;
use moka::future::Cache;
use thiserror::Error;
use uuid::Uuid;
use crate::common_utils::cache_utils::get_or_fetch_entity;

use crate::common_utils::dao_error::DaoError;
use crate::masters::business_entity_master::business_entity_dao::{BusinessEntityDao, get_business_entity_dao};
use crate::masters::business_entity_master::business_entity_models::{BusinessEntityMaster, CreateBusinessEntityRequest};

#[derive(Debug, Error)]
pub enum BusinessEntityServiceError {
    #[error(transparent)]
    Db(#[from] DaoError)
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait BusinessEntityService: Send + Sync {
    async fn get_business_entity_by_id(&self, id: &Uuid, tenant_id: &Uuid) ->
    Result<Option<Arc<BusinessEntityMaster>>, BusinessEntityServiceError>;

    async fn is_valid_business_entity_id(&self, id: &Uuid, tenant_id: &Uuid) ->
    Result<bool, BusinessEntityServiceError>;
    async fn create_business_entity(&self, request: &CreateBusinessEntityRequest)
                                    -> Result<Uuid, BusinessEntityServiceError>;
}


struct BusinessEntityServiceImpl {
    dao: Arc<dyn BusinessEntityDao>,
    //tenant_id, business_entity_id
    //lets keep the size to be 1000 and ttl to be 5 minutes
    cache_id: Cache<(Uuid, Uuid), Arc<BusinessEntityMaster>>,
}

#[allow(dead_code)]
pub fn get_business_entity_master_service(arc: Arc<Pool>) -> Arc<dyn BusinessEntityService> {
    let dao = get_business_entity_dao(arc);
    let cache: Cache<(Uuid, Uuid), Arc<BusinessEntityMaster>> = Cache::builder()
        .max_capacity(1000)
        .time_to_live(Duration::from_secs(300))
        .build();
    let service = BusinessEntityServiceImpl {
        dao,
        cache_id: cache,
    };
    Arc::new(service)
}


#[async_trait]
impl BusinessEntityService for BusinessEntityServiceImpl {
    async fn get_business_entity_by_id(&self, id: &Uuid, tenant_id: &Uuid) -> Result<Option<Arc<BusinessEntityMaster>>, BusinessEntityServiceError> {
        get_or_fetch_entity(*tenant_id, *id,
                            &self.cache_id,
                            async {
                                let p = self.dao.get_business_entity(id, tenant_id).await?;
                                Ok(p)
                            }).await
    }

    async fn is_valid_business_entity_id(&self, id: &Uuid, tenant_id: &Uuid) -> Result<bool, BusinessEntityServiceError> {
        let entity = self.get_business_entity_by_id(id, tenant_id).await?;
        return Ok(entity.is_some());
    }

    async fn create_business_entity(&self, request: &CreateBusinessEntityRequest) -> Result<Uuid, BusinessEntityServiceError> {
        let kk = self.dao.create_business_entity(request).await?;
        Ok(kk)
    }
}

