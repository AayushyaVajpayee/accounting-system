use async_trait::async_trait;
use deadpool_postgres::Pool;
#[cfg(test)]
use mockall::automock;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::tenant::tenant_dao::{get_tenant_dao, TenantDao};
use crate::tenant::tenant_models::{CreateTenantRequest, Tenant};

#[derive(Debug, Error)]
pub enum TenantServiceError {
    #[error("validation failures \n {}",.0.join("\n"))]
    Validation(Vec<String>), //4xx
    #[error(transparent)]
    Db(#[from] DaoError), //5xx
    //have to separate out idempotency check
    #[error("{0}")]
    Other(String),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait TenantService: Send + Sync {
    async fn get_tenant_by_id(&self, id: Uuid) -> Result<Option<Tenant>, TenantServiceError>;
    async fn create_tenant(&self, tenant: &CreateTenantRequest)
        -> Result<Uuid, TenantServiceError>;
}

struct TenantServiceImpl {
    tenant_dao: Arc<dyn TenantDao>,
}

#[async_trait]
impl TenantService for TenantServiceImpl {
    async fn get_tenant_by_id(&self, id: Uuid) -> Result<Option<Tenant>, TenantServiceError> {
        //todo to be cached locally
        let tenant = self.tenant_dao.get_tenant_by_id(id).await?;
        Ok(tenant)
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
    let tenant_s = TenantServiceImpl {
        tenant_dao: tenant_d,
    };
    Arc::new(tenant_s)
}