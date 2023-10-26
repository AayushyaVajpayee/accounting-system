use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use crate::accounting::postgres_factory::get_postgres_conn_pool;
use crate::tenant::tenant_dao::{get_tenant_dao, TenantDao};
use crate::tenant::tenant_models::{CreateTenantRequest, Tenant};
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait TenantService:Send+Sync {
    async fn get_tenant_by_id(&self, id: Uuid) -> Option<Tenant>;
    async fn create_tenant(&self, tenant: &CreateTenantRequest) -> Uuid;
}

struct TenantServiceImpl {
    tenant_dao: Arc<dyn TenantDao>
}

#[async_trait]
impl TenantService for TenantServiceImpl{
    async fn get_tenant_by_id(&self, id: Uuid) -> Option<Tenant> {
        //todo to be cached locally
        self.tenant_dao.get_tenant_by_id(id).await
    }

    async fn create_tenant(&self, tenant: &CreateTenantRequest) -> Uuid {
        self.tenant_dao.create_tenant(tenant).await
    }
}

pub fn get_tenant_service() -> Arc<dyn TenantService> {
    let pclient = get_postgres_conn_pool();
    let tenant_d=get_tenant_dao(pclient);
    let tenant_s=TenantServiceImpl{tenant_dao:tenant_d};
    Arc::new(tenant_s)

}

#[allow(dead_code)]
#[cfg(test)]
pub fn get_tenant_service_for_test(postgres_client: &'static deadpool_postgres::Pool) -> Arc<dyn TenantService> {
    let tenant_d=get_tenant_dao(postgres_client);
    let tenant_s=TenantServiceImpl{tenant_dao:tenant_d};
    Arc::new(tenant_s)
}