use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use moka::future::Cache;
use thiserror::Error;
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::masters::business_entity_master::business_entity_dao::BusinessEntityDao;
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


#[async_trait]
impl BusinessEntityService for BusinessEntityServiceImpl {
    async fn get_business_entity_by_id(&self, id: &Uuid, tenant_id: &Uuid) -> Result<Option<Arc<BusinessEntityMaster>>, BusinessEntityServiceError> {
        let key = (*id, *tenant_id);
        if let Some(entity) = self.cache_id.get(&key).await {
            return Ok(Some(entity));
        }
        let kk = self.dao.get_business_entity(id, tenant_id).await?;
        if let Some(en) = kk {
            let k = Arc::new(en);
            self.cache_id.insert(key, k.clone()).await;
            Ok(Some(k))
        } else {
            Ok(None)
        }
    }

    async fn is_valid_business_entity_id(&self, id: &Uuid, tenant_id: &Uuid) -> Result<bool, BusinessEntityServiceError> {
        let entity = self.get_business_entity_by_id(id, tenant_id).await?;
        return Ok(entity.is_some())
    }

    async fn create_business_entity(&self, request: &CreateBusinessEntityRequest) -> Result<Uuid, BusinessEntityServiceError> {
        let kk = self.dao.create_business_entity(request).await?;
        Ok(kk)
    }
}

