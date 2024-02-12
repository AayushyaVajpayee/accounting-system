use async_trait::async_trait;
use deadpool_postgres::Pool;
#[cfg(test)]
use mockall::automock;
use std::sync::Arc;
use std::time::Duration;
use moka::future::Cache;
use thiserror::Error;
use uuid::Uuid;
use crate::common_utils::cache_utils::get_or_fetch_entity;

use crate::common_utils::dao_error::DaoError;
use crate::tenant::tenant_dao::{get_tenant_dao, TenantDao};
use crate::tenant::tenant_models::{CreateTenantRequest, Tenant};

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum TenantServiceError {
    #[error("validation failures \n {}", .0.join("\n"))]
    Validation(Vec<String>),
    //4xx
    #[error(transparent)]
    Db(#[from] DaoError),
    //5xx
    //have to separate out idempotency check
    #[error("{0}")]
    Other(String),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait TenantService: Send + Sync {
    async fn get_tenant_by_id(&self, id: Uuid) -> Result<Option<Arc<Tenant>>, TenantServiceError>;
    async fn create_tenant(&self, tenant: &CreateTenantRequest)
                           -> Result<Uuid, TenantServiceError>;
}

struct TenantServiceImpl {
    tenant_dao: Arc<dyn TenantDao>,
    cache_by_id: Cache<(Uuid, Uuid), Arc<Tenant>>
}

#[async_trait]
impl TenantService for TenantServiceImpl {
    async fn get_tenant_by_id(&self, id: Uuid) -> Result<Option<Arc<Tenant>>, TenantServiceError> {
        let fetch = async
            {
                let p = self.tenant_dao
                    .get_tenant_by_id(id).await?;
                Ok(p)
            };
        get_or_fetch_entity(id, id, &self.cache_by_id,
                            fetch).await
    }

    async fn create_tenant(
        &self,
        tenant: &CreateTenantRequest,
    ) -> Result<Uuid, TenantServiceError> {
        let tenant_id = self.tenant_dao.create_tenant(tenant).await?;
        Ok(tenant_id)
    }
}

pub fn get_tenant_service(arc: Arc<Pool>) -> Arc<dyn TenantService> {
    let tenant_d = get_tenant_dao(arc);
    let cache: Cache<(Uuid, Uuid), Arc<Tenant>> = Cache::builder()
        .max_capacity(500)
        .time_to_live(Duration::from_secs(300))
        .build();
    let tenant_s = TenantServiceImpl {
        tenant_dao: tenant_d,
        cache_by_id: cache,
    };
    Arc::new(tenant_s)
}
