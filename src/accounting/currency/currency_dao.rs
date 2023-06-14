use std::error::Error;

use postgres::Client;
use postgres::types::ToSql;

use crate::accounting::currency::currency_models::{AuditMetadataBase, CurrencyMaster};

pub trait CurrencyDao {
    fn get_currency_entry_by_id(&mut self, id: &i16) -> Option<CurrencyMaster>;
    fn create_currency_entry(&mut self, currency: CurrencyMaster) -> i16;
    fn update_currency_entry();
    fn delete_currency_entry();
}

pub struct CurrencyDaoPostgresImpl {
    postgres_client: Client,
}

impl CurrencyDao for CurrencyDaoPostgresImpl {
    fn get_currency_entry_by_id(&mut self, id: &i16) -> Option<CurrencyMaster> {
        let k = self.postgres_client.
            query("select id,tenant_id,scale,display_name,description,created_by,updated_by,\
            created_at,updated_at from currency_master where id = $1", &[id]).unwrap();
        k.iter().map(|row|
            CurrencyMaster {
                id: row.get(0),
                tenant_id: row.get(1),
                scale: row.get(2),
                display_name: row.get(3),
                description: row.get(4),
                audit_metadata: AuditMetadataBase {
                    created_by: row.get(5),
                    updated_by: row.get(6),
                    created_at: row.get(7),
                    updated_at: row.get(8),
                },
            }).next()
    }

    fn create_currency_entry(&mut self, currency: CurrencyMaster) -> i16 {
        self.postgres_client.query(
            "insert into currency_master  (tenant_id,scale,display_name,description,created_by,updated_by,
             created_at,updated_at) values ($1,$2,$3,$4,$5,$6,$7,$8) returning id
            ",
            &[&(currency.tenant_id), &(currency.scale), &currency.display_name,
                &currency.description,
                &currency.audit_metadata.created_by, &currency.audit_metadata.updated_by,
                &(currency.audit_metadata.created_at), &(currency.audit_metadata.updated_at)],
        ).unwrap().iter().map(|row| row.get(0)).next().unwrap()
        // self.postgres_client.simple_query(&query).unwrap();
    }

    fn update_currency_entry() {
        todo!()
    }

    fn delete_currency_entry() {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use postgres::{Client, NoTls};
    use testcontainers::clients;
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;

    use crate::accounting::currency::currency_dao::{CurrencyDao, CurrencyDaoPostgresImpl};
    use crate::accounting::currency::currency_models::{a_currency_master, CurrencyMasterTestBuilder};
    use crate::accounting::tenant::tenant_models::a_create_tenant_request;
    use crate::accounting::tenant::tenant_service::get_tenant_service_for_test;

    fn create_postgres_client(port: u16) -> Client {
        let con_str =
            format!("host=localhost user=postgres password=postgres dbname=postgres port={port}");
        let client = Client::
        connect(&con_str, NoTls)
            .unwrap();
        client
    }

    fn create_schema(client: &mut Client) {
        let path = format!("schema/postgres/schema.sql");
        let fi = std::fs::read_to_string(path).unwrap();
        // println!("{fi}");
        client.simple_query(&fi).unwrap();
    }

    #[test]
    fn test_prep() {
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
        create_schema(&mut postgres_client);
        let mut currency_dao = CurrencyDaoPostgresImpl { postgres_client };
        let tenant_postgres = create_postgres_client(port);
        let mut tenant_service = get_tenant_service_for_test(tenant_postgres);
        let a_tenant = a_create_tenant_request(Default::default());
       let tenant_id= tenant_service.create_tenant(&a_tenant);
        let currency_master = a_currency_master(CurrencyMasterTestBuilder {
            tenant_id: Some(tenant_id),
            ..Default::default()
        }
        );
        currency_dao.create_currency_entry(currency_master);
        let got_c = currency_dao.get_currency_entry_by_id(&1);
        println!("{:?}", got_c)
    }
}