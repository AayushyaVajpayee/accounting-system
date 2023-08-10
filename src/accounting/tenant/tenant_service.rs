use async_trait::async_trait;
use deadpool_postgres::Pool;
use crate::accounting::postgres_factory::get_postgres_conn_pool;
use crate::accounting::tenant::tenant_dao::{get_tenant_dao, TenantDao};
use crate::accounting::tenant::tenant_models::{CreateTenantRequest, Tenant};

#[async_trait]
pub trait TenantService {
    async fn get_tenant_by_id(&self, id: &i32) -> Option<Tenant>;
    async fn create_tenant(&self, tenant: &CreateTenantRequest) -> i32;
}

struct TenantServiceImpl {
    tenant_dao: Box<dyn TenantDao + Send + Sync>
}

#[async_trait]
impl TenantService for TenantServiceImpl{
    async fn get_tenant_by_id(&self, id: &i32) -> Option<Tenant> {
        self.tenant_dao.get_tenant_by_id(id).await
    }

    async fn create_tenant(&self, tenant: &CreateTenantRequest) -> i32 {
        self.tenant_dao.create_tenant(tenant).await
    }
}

pub fn get_tenant_service() -> Box<dyn TenantService + Send + Sync> {
    let pclient = get_postgres_conn_pool();
    let tenant_d=get_tenant_dao(pclient);
    let tenant_s=TenantServiceImpl{tenant_dao:tenant_d};
    Box::new(tenant_s)

}

#[allow(dead_code)]
#[cfg(test)]
pub fn get_tenant_service_for_test(postgres_client: &'static Pool) -> Box<dyn TenantService + Send + Sync> {
    let tenant_d=get_tenant_dao(postgres_client);
    let tenant_s=TenantServiceImpl{tenant_dao:tenant_d};
    Box::new(tenant_s)
}