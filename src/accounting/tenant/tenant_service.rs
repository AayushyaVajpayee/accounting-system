use postgres::Client;
use crate::accounting::tenant::tenant_dao::{get_tenant_dao, TenantDao};
use crate::accounting::tenant::tenant_models::{CreateTenantRequest, Tenant};
use  crate::accounting::postgres_factory::create_postgres_client;
pub trait TenantService{
    fn get_tenant_by_id(&mut self,id:&i32)->Option<Tenant>;
    fn create_tenant(&mut self,tenant:&CreateTenantRequest)->i32;

}
struct TenantServiceImpl{
    tenant_dao: Box<dyn TenantDao>
}
impl TenantService for TenantServiceImpl{
    fn get_tenant_by_id(&mut self, id: &i32) -> Option<Tenant> {
        self.tenant_dao.get_tenant_by_id(id)
    }

    fn create_tenant(&mut self, tenant: &CreateTenantRequest) -> i32 {
        self.tenant_dao.create_tenant(&tenant)
    }
}

pub fn get_tenant_service()-> Box<dyn TenantService> {
    let pclient=create_postgres_client();
    let tenant_d=get_tenant_dao(pclient);
    let tenant_s=TenantServiceImpl{tenant_dao:tenant_d};
    Box::new(tenant_s)

}

#[cfg(test)]
pub fn get_tenant_service_for_test(postgres_client:Client) -> Box<dyn TenantService>{
    let tenant_d=get_tenant_dao(postgres_client);
    let tenant_s=TenantServiceImpl{tenant_dao:tenant_d};
    Box::new(tenant_s)
}