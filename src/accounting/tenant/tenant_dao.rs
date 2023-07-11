use postgres::Client;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::accounting::tenant::tenant_models::{CreateTenantRequest, Tenant};

pub trait TenantDao {
    fn get_tenant_by_id(&mut self, id: &i32) -> Option<Tenant>;
    fn create_tenant(&mut self, tenant: &CreateTenantRequest) -> i32;
    fn update_tenant(&mut self, tenant: &CreateTenantRequest) -> i64;
    fn delete_tenant(&mut self, tenant_id: &str) -> i64;
}

pub fn get_tenant_dao(client:Client)->Box<dyn TenantDao>{
    let td=TenantDaoImpl{
        postgres_client:client
    };
    Box::new(td)
}

 struct TenantDaoImpl {
    postgres_client: Client,
}

impl TenantDao for TenantDaoImpl {
    fn get_tenant_by_id(&mut self, id: &i32) -> Option<Tenant> {
        let k = self.postgres_client
            .query("select id,display_name,created_by,updated_by,created_at,updated_at
            from tenant where id =$1"
                   , &[id])
            .unwrap();

        k.iter().map(|row|
            Tenant {
                id: row.get(0),
                display_name: row.get(1),
                audit_metadata: AuditMetadataBase {
                    created_by: row.get(2),
                    updated_by: row.get(3),
                    created_at: row.get(4),
                    updated_at: row.get(5),
                },
            }
        ).next()
    }

    fn create_tenant(&mut self, tenant: &CreateTenantRequest) -> i32 {
        self.postgres_client.query(
            "insert into tenant (display_name,created_by,updated_by,created_at,updated_at)
            values ($1,$2,$3,$4,$5) returning id", &[
                &tenant.display_name,
                &tenant.audit_metadata.created_by,
                &tenant.audit_metadata.updated_by,
                &tenant.audit_metadata.created_at,
                &tenant.audit_metadata.updated_at
            ],
        ).unwrap().iter().map(|row| row.get(0)).next().unwrap()
    }

    fn update_tenant(&mut self, tenant: &CreateTenantRequest) -> i64 {
        todo!()
    }

    fn delete_tenant(&mut self, tenant_id: &str) -> i64 {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use crate::accounting::tenant::tenant_dao::{TenantDao, TenantDaoImpl};
    use crate::accounting::tenant::tenant_models::a_create_tenant_request;
    use crate::test_utils::test_utils_postgres::{create_postgres_client, get_postgres_image_port};

    #[test]
    fn test_tenant() {
        let port = get_postgres_image_port();
        let mut postgres_client = create_postgres_client(port);
        let t1 = a_create_tenant_request(Default::default());
        let mut tenant_dao = TenantDaoImpl { postgres_client };
        tenant_dao.create_tenant(&t1);
        let created_tenant_id = tenant_dao.create_tenant(&t1);
        // println!("created {} tenant", tenant_dao.create_tenant(tenant));
        println!("fetched {:?}", tenant_dao.get_tenant_by_id(&created_tenant_id));
        // panic!("kkjkj");
    }
}