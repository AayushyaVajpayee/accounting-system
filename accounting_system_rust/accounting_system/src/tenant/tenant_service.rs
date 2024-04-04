use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use deadpool_postgres::Pool;
use lazy_static::lazy_static;
#[cfg(test)]
use mockall::automock;
use moka::future::Cache;
use thiserror::Error;
use uuid::Uuid;

use crate::common_utils::cache_utils::get_or_fetch_entity;
use crate::common_utils::dao_error::DaoError;
use crate::tenant::tenant_dao::{get_tenant_dao, TenantDao};
use crate::tenant::tenant_models::{CreateTenantRequest, Tenant};

lazy_static! {
    pub static ref SUPER_TENANT_ID:Uuid= Uuid::from_str("018b33d9-c862-7fde-a0cd-55504d75e5e9").unwrap();
}

lazy_static! {
    pub static ref SUPER_USER_ID: Uuid =
        Uuid::from_str("018b3444-dc75-7a3f-a4d9-02c41071d3bd").unwrap();
}
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
    async fn create_tenant(&self, tenant: &CreateTenantRequest, tenant_id: Uuid,
                           user_id: Uuid)
                           -> Result<Uuid, TenantServiceError>;
}

struct TenantServiceImpl {
    tenant_dao: Arc<dyn TenantDao>,
    cache_by_id: Cache<(Uuid, Uuid), Arc<Tenant>>,
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
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Uuid, TenantServiceError> {
        let mut validation: Vec<String> = Vec::new();
        if tenant_id != *SUPER_TENANT_ID {
            validation.push(format!("tenant_id {} is not authorised to create new tenant", tenant_id));
        }
        if user_id != *SUPER_USER_ID {
            validation.push(format!("user id {} is not authorised to create new tenant", user_id));
        }
        if !validation.is_empty() {
            return Err(TenantServiceError::Validation(validation));
        }
        let tenant_id = self.tenant_dao.create_tenant(tenant, user_id).await?;
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
