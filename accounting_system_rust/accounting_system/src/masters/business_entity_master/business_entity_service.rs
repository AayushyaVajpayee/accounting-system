use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
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
    Result<Option<BusinessEntityMaster>, BusinessEntityServiceError>;

    async fn is_valid_business_entity_id(&self, id: &Uuid, tenant_id: &Uuid)->
                                                                             Result<bool,BusinessEntityServiceError>;
    async fn create_business_entity(&self, request: &CreateBusinessEntityRequest)
                                    -> Result<Uuid, BusinessEntityServiceError>;
}



struct BusinessEntityServiceImpl {
    dao: Arc<dyn BusinessEntityDao>,
}


#[async_trait]
impl BusinessEntityService for BusinessEntityServiceImpl {
    async fn get_business_entity_by_id(&self, id: &Uuid, tenant_id: &Uuid) -> Result<Option<BusinessEntityMaster>, BusinessEntityServiceError> {
        let kk = self.dao.get_business_entity(id, tenant_id).await?;
        Ok(kk)
    }

    async fn is_valid_business_entity_id(&self, id: &Uuid, tenant_id: &Uuid) -> Result<bool, BusinessEntityServiceError> {
        let k  = self.dao.is_business_entity_exist(id,tenant_id).await?;
        Ok(k)

    }

    async fn create_business_entity(&self, request: &CreateBusinessEntityRequest) -> Result<Uuid, BusinessEntityServiceError> {
        let kk = self.dao.create_business_entity(request).await?;
        Ok(kk)
    }


}

