use std::sync::OnceLock;

use postgres::Client;

use crate::accounting::account::account_models::{AccountTypeMaster, CreateAccountTypeMasterRequest};
use crate::accounting::currency::currency_models::AuditMetadataBase;

pub trait AccountTypeDao {
    fn get_account_type_by_id(&mut self, id: &i16) -> Option<AccountTypeMaster>;
    fn create_account_type(&mut self, request: &CreateAccountTypeMasterRequest) -> i16;
    fn get_all_account_types_for_tenant_id(&mut self, tenant_id: &i32) -> Vec<AccountTypeMaster>;
}

pub struct AccountTypeDaoPostgresImpl {
    postgres_client: Client,
}

const ACCOUNT_TYPE_POSTGRES_SELECT_FIELDS: &str =
    "id,tenant_id,display_name,account_code,created_by,updated_by,created_at,updated_at";
const ACCOUNT_TYPE_TABLE_NAME: &str = "account_type_master";
static ACCOUNT_TYPE_BY_ID_QUERY: OnceLock<String> = OnceLock::new();
static ACCOUNT_TYPE_INSERT_STATEMENT: OnceLock<String> = OnceLock::new();

impl AccountTypeDaoPostgresImpl {
    fn get_account_type_by_id_query() -> &'static String {
        ACCOUNT_TYPE_BY_ID_QUERY.get_or_init(|| {
            format!("select {ACCOUNT_TYPE_POSTGRES_SELECT_FIELDS} from {ACCOUNT_TYPE_TABLE_NAME} where id=$1")
        })
    }

    fn create_insert_statement() -> &'static String {
        ACCOUNT_TYPE_INSERT_STATEMENT.get_or_init(|| {
            format!("insert into {ACCOUNT_TYPE_TABLE_NAME} ({ACCOUNT_TYPE_POSTGRES_SELECT_FIELDS})\
            values (DEFAULT,$1,$2,$3,$4,$5,$6,$7) returning id")
        })
    }
}


impl AccountTypeDao for AccountTypeDaoPostgresImpl {
    fn get_account_type_by_id(&mut self, id: &i16) -> Option<AccountTypeMaster> {
        let query = AccountTypeDaoPostgresImpl::get_account_type_by_id_query();
        let k = self.postgres_client
            .query(query,
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
        let query = AccountTypeDaoPostgresImpl::create_insert_statement();
        self.postgres_client.query(
            query,
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

    fn get_all_account_types_for_tenant_id(&mut self, tenant_id: &i32) -> Vec<AccountTypeMaster> {
        // let k = self.postgres_client.query(
        //     "select"
        // )
        todo!()
    }
}


#[cfg(test)]
mod account_type_tests {
    use postgres::{Client, NoTls};
    use testcontainers::clients;
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;

    use crate::accounting::account::account_models::{a_create_account_type_master_request, CreateAccountTypeMasterRequestTestBuilder};
    use crate::accounting::account::account_type_dao::{AccountTypeDao, AccountTypeDaoPostgresImpl};
    use crate::seeddata::seed_service::copy_tables;

    fn create_postgres_client(port: u16) -> Client {
        let con_str =
            format!("host=localhost user=postgres password=postgres dbname=postgres port={port}");
        Client::
        connect(&con_str, NoTls)
            .unwrap()
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
        copy_tables(port);
        let mut account_type_dao = AccountTypeDaoPostgresImpl {
            postgres_client: create_postgres_client(port)
        };
        let an_account_type = a_create_account_type_master_request(
            CreateAccountTypeMasterRequestTestBuilder {
                tenant_id: Some(1),
                ..Default::default()
            });
        let account_type_id = account_type_dao.create_account_type(&an_account_type);
        let _ = account_type_dao
            .get_account_type_by_id(&account_type_id)
            .unwrap();
    }
}
