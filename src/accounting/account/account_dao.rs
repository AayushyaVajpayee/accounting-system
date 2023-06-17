use postgres::Client;

use crate::accounting::account::account_models::{AccountTypeMaster, CreateAccountTypeMasterRequest};
use crate::accounting::currency::currency_models::AuditMetadataBase;

pub trait AccountTypeDao {
    fn get_account_type_by_id(&mut self, id: &i16) -> Option<AccountTypeMaster>;
    fn create_account_type(&mut self, request: &CreateAccountTypeMasterRequest) -> i16;
}

pub struct AccountTypeDaoPostgresImpl {
    postgres_client: Client,
}

impl AccountTypeDao for AccountTypeDaoPostgresImpl {
    fn get_account_type_by_id(&mut self, id: &i16) -> Option<AccountTypeMaster> {
        let k = self.postgres_client
            .query("select id,tenant_id,display_name,account_code,created_by,updated_by,created_at,updated_at
            from account_type_master where id = $1",
                   &[id]).unwrap();
        k.iter().map(|row|
            AccountTypeMaster {
                id: row.get(0),
                tenant_id: row.get(1),
                display_name: row.get(2),
                account_code: row.get(3),
                audit_metadata: AuditMetadataBase {
                    created_by: row.get(4),
                    updated_by: row.get(5),
                    created_at: row.get(6),
                    updated_at: row.get(7),
                },
            }
        ).next()
    }

    fn create_account_type(&mut self, request: &CreateAccountTypeMasterRequest) -> i16 {
        self.postgres_client.query(
            "insert into account_type_master (tenant_id,display_name,account_code,created_by,updated_by,created_at,updated_at)
          values ($1,$2,$3,$4,$5,$6,$7) returning id
          ",
            &[
                &request.tenant_id,
                &request.display_name,
                &request.account_code,
                &request.audit_metadata.created_by,
                &request.audit_metadata.updated_by,
                &request.audit_metadata.created_at,
                &request.audit_metadata.updated_at
            ],
        ).unwrap()
            .iter()
            .map(|row| row.get(0))
            .next()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use postgres::{Client, NoTls};
    use testcontainers::clients;
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;

    use crate::accounting::account::account_dao::{AccountTypeDao, AccountTypeDaoPostgresImpl};
    use crate::accounting::account::account_models::{a_create_account_type_master_request, CreateAccountTypeMasterRequestTestBuilder};
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
    fn tests() {
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
        let mut tenant_service = get_tenant_service_for_test(postgres_client);
        let a_tenant = a_create_tenant_request(Default::default());
        let tenant_id = tenant_service.create_tenant(&a_tenant);
        // let mut currency_postgres_client = create_postgres_client(port);
        // let mut currency_service =
        //     get_currency_service_for_test(currency_postgres_client);

        // let currency_master = a_create_currency_master_request(
        //     CreateCurrencyMasterRequestTestBuilder{
        //     tenant_id:Some(tenant_id),
        //     ..Default::default()
        // });
        // let currency_master_id= currency_service.create_currency_entry(&currency_master);
        let mut account_type_dao = AccountTypeDaoPostgresImpl {
            postgres_client: create_postgres_client(port)
        };
        let an_account_type = a_create_account_type_master_request(
            CreateAccountTypeMasterRequestTestBuilder {
                tenant_id: Some(tenant_id),
                ..Default::default()
            });
        let account_type_id = account_type_dao.create_account_type(&an_account_type);
        let account_type = account_type_dao
            .get_account_type_by_id(&account_type_id)
            .unwrap();
    }
}