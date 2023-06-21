use postgres::Client;

use crate::accounting::account::account_models::{Account, AccountTypeMaster, CreateAccountRequest, CreateAccountTypeMasterRequest};
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

pub trait AccountDao {
    fn get_account_by_id(&mut self, id: &i32) -> Option<Account>;
    fn create_account(&mut self, request: &CreateAccountRequest) -> i32;
}

pub struct AccountDaoPostgresImpl {
    postgres_client: Client,
}

impl AccountDao for AccountDaoPostgresImpl {
    fn get_account_by_id(&mut self, id: &i32) -> Option<Account> {
        let k = self.postgres_client.query(
            "select id,tenant_id,display_code,account_type_id,\
            user_id,ledger_master_id,debits_posted,debits_pending,credits_posted,credits_pending,\
            created_by,updated_by,created_at,updated_at \
            from user_account where id=$1",
            &[id],
        ).unwrap();
        k.iter()
            .map(|row|
                Account {
                    id: row.get(0),
                    tenant_id: row.get(1),
                    display_code: row.get(2),
                    account_type_id: row.get(3),
                    user_id: row.get(4),
                    ledger_master_id: row.get(5),
                    debits_posted: row.get(6),
                    debits_pending: row.get(7),
                    credits_posted: row.get(8),
                    credits_pending: row.get(9),
                    audit_metadata: AuditMetadataBase {
                        created_by: row.get(10),
                        updated_by: row.get(11),
                        created_at: row.get(12),
                        updated_at: row.get(13),
                    },
                }).next()
    }

    fn create_account(&mut self, request: &CreateAccountRequest) -> i32 {
        self.postgres_client.query(
            "insert into user_account (tenant_id,display_code,account_type_id,\
            user_id,ledger_master_id,debits_posted,debits_pending,credits_posted,credits_pending,\
            created_by,updated_by,created_at,updated_at)
            values ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) returning id
            ",
            &[
                &request.tenant_id,
                &request.display_code,
                &request.account_type_id,
                &request.user_id,
                &request.ledger_master_id,
                &0, &0, &0, &0,
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
mod account_type_tests {
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

#[cfg(test)]
mod account_tests {
    use postgres::{Client, NoTls};
    use testcontainers::clients;
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;

    use crate::accounting::account::account_dao::{AccountDao, AccountDaoPostgresImpl, AccountTypeDao, AccountTypeDaoPostgresImpl};
    use crate::accounting::account::account_models::{a_create_account_request, a_create_account_type_master_request, CreateAccountRequestTestBuilder, CreateAccountTypeMasterRequestTestBuilder};
    use crate::accounting::currency::currency_models::{a_create_currency_master_request, CreateCurrencyMasterRequestTestBuilder};
    use crate::accounting::currency::currency_service::get_currency_service_for_test;
    use crate::accounting::tenant::tenant_models::a_create_tenant_request;
    use crate::accounting::tenant::tenant_service::get_tenant_service_for_test;
    use crate::accounting::user::user_models::{a_create_user_request, CreateUserRequestTestBuilder};
    use crate::accounting::user::user_service::get_user_service_for_test;

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
    fn test_account() {
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
        // let mut currency_dao = CurrencyDaoPostgresImpl { postgres_client };
        let tenant_postgres = create_postgres_client(port);
        let mut tenant_service = get_tenant_service_for_test(tenant_postgres);
        let a_tenant = a_create_tenant_request(Default::default());
        let tenant_id = tenant_service.create_tenant(&a_tenant);
        let mut currency_service = get_currency_service_for_test(create_postgres_client(port));
        let currency_creation_request = a_create_currency_master_request(
            CreateCurrencyMasterRequestTestBuilder {
                tenant_id: Some(tenant_id),
                ..Default::default()
            }
        );
        let currency_id = currency_service.create_currency_entry(&currency_creation_request);
        let mut account_type_dao = AccountTypeDaoPostgresImpl {
            postgres_client: create_postgres_client(port)
        };
        let an_account_type = a_create_account_type_master_request(CreateAccountTypeMasterRequestTestBuilder {
            tenant_id: Some(tenant_id),
            ..Default::default()
        });
        let account_type_id = account_type_dao.create_account_type(&an_account_type);
        let mut account_dao = AccountDaoPostgresImpl {
            postgres_client: create_postgres_client(port)
        };
        let mut user_service = get_user_service_for_test(create_postgres_client(port));
        let user_creation_request = a_create_user_request(CreateUserRequestTestBuilder {
            tenant_id: Some(tenant_id),
            ..Default::default()
        });
        let user_id = user_service.create_user(&user_creation_request);
        let an_account_request = a_create_account_request(CreateAccountRequestTestBuilder {
            tenant_id: Some(tenant_id),
            ledger_master_id: Some(todo!()),
            account_type_id: Some(account_type_id),
            user_id: Some(user_id),
            ..Default::default()
        });
        let account_id = account_dao.create_account(&an_account_request);
        let account_fetched = account_dao.get_account_by_id(&account_id).unwrap();
    }
}