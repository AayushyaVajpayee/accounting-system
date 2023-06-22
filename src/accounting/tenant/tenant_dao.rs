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
    use postgres::{Client, NoTls};
    use testcontainers::clients;
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;

    use crate::accounting::tenant::tenant_dao::{TenantDao, TenantDaoImpl};
    use crate::accounting::tenant::tenant_models::a_create_tenant_request;
    use crate::seeddata::seed_service::copy_tables;

    fn create_postgres_client(port: u16) -> Client {
        let con_str =
            format!("host=localhost user=postgres password=postgres dbname=postgres port={port}");
        let client = Client::
        connect(&con_str, NoTls)
            .unwrap();
        client
    }

    #[test]
    fn test_tenant() {
        let test_container_client = clients::Cli::default();
        let image = "postgres";
        let image_tag = "latest";
        let generic_postgres = GenericImage::new(image, image_tag)
            .with_wait_for(WaitFor::message_on_stderr("database system is ready to accept connections"))
            .with_env_var("POSTGRES_DB", "postgres")
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres");
        let node = test_container_client.run(generic_postgres);
        let port = node.get_host_port_ipv4(5432);
        let mut postgres_client = create_postgres_client(port);
        copy_tables(port);
        let t1=a_create_tenant_request(Default::default());
        let mut tenant_dao = TenantDaoImpl { postgres_client };
        tenant_dao.create_tenant(&t1);
        let created_tenant_id = tenant_dao.create_tenant(&t1);
        // println!("created {} tenant", tenant_dao.create_tenant(tenant));
        println!("fetched {:?}", tenant_dao.get_tenant_by_id(&created_tenant_id));
        // panic!("kkjkj");
    }
}