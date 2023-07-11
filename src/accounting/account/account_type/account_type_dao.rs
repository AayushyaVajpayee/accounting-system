use std::sync::OnceLock;

use postgres::{Client, Row};

use crate::accounting::account::account_type::account_type_models::{AccountTypeMaster, CreateAccountTypeMasterRequest};
use crate::accounting::currency::currency_models::AuditMetadataBase;

pub trait AccountTypeDao {
    fn get_account_type_by_id(&mut self, id: &i16) -> Option<AccountTypeMaster>;
    fn create_account_type(&mut self, request: &CreateAccountTypeMasterRequest) -> i16;
    fn get_all_account_types_for_tenant_id(&mut self, tenant_id: &i32) -> Vec<AccountTypeMaster>;
}

pub struct AccountTypeDaoPostgresImpl {
    postgres_client: Client,
}

const SELECT_FIELDS: &str =
    "id,tenant_id,child_ids,parent_id,display_name,account_code,created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "account_type_master";
static BY_ID_QUERY: OnceLock<String> = OnceLock::new();
static INSERT_STATEMENT: OnceLock<String> = OnceLock::new();
static ALL_TYPES_FOR_TENANT: OnceLock<String> = OnceLock::new();

impl AccountTypeDaoPostgresImpl {
    fn get_account_type_by_id_query() -> &'static String {
        BY_ID_QUERY.get_or_init(|| {
            format!("select {SELECT_FIELDS} from {TABLE_NAME} where id=$1")
        })
    }

    fn create_insert_statement() -> &'static String {
        INSERT_STATEMENT.get_or_init(|| {
            format!("insert into {TABLE_NAME} ({SELECT_FIELDS})\
            values (DEFAULT,$1,$2,$3,$4,$5,$6,$7,$8,$9) returning id")
        })
    }

    fn get_all_types_for_tenant_id_query() -> &'static String {
        ALL_TYPES_FOR_TENANT.get_or_init(|| {
            format!("select {SELECT_FIELDS} from {TABLE_NAME} where tenant_id=$1")
        })
    }
}

impl TryFrom<&Row> for AccountTypeMaster {
    type Error = ();

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(AccountTypeMaster {
            id: row.get(0),
            tenant_id: row.get(1),
            child_ids: row.try_get(2).ok(),
            parent_id: row.try_get(3).ok(),
            display_name: row.get(4),
            account_code: row.get(5),
            audit_metadata: AuditMetadataBase {
                created_by: row.get(6),
                updated_by: row.get(7),
                created_at: row.get(8),
                updated_at: row.get(9),
            },
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
            row.try_into().unwrap()
        ).next()
    }

    fn create_account_type(&mut self, request: &CreateAccountTypeMasterRequest) -> i16 {
        let query = AccountTypeDaoPostgresImpl::create_insert_statement();
        self.postgres_client.query(
            query,
            &[
                &request.tenant_id,
                &request.child_ids,
                &request.parent_id,
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
        let query = AccountTypeDaoPostgresImpl::get_all_types_for_tenant_id_query();
        self.postgres_client.query(query, &[tenant_id])
            .unwrap().iter().map(|row| row.try_into().unwrap()).collect()
    }
}


#[cfg(test)]
mod account_type_tests {
    use postgres::{Client, NoTls};

    use crate::accounting::account::account_type::account_type_dao::{AccountTypeDao, AccountTypeDaoPostgresImpl};
    use crate::accounting::account::account_type::account_type_models::{a_create_account_type_master_request, CreateAccountTypeMasterRequestTestBuilder};
    use crate::seeddata::seed_service::copy_tables;
    use crate::test_utils::test_utils_postgres::run_postgres;

    fn create_postgres_client(port: u16) -> Client {
        let con_str =
            format!("host=localhost user=postgres password=postgres dbname=postgres port={port}");
        Client::
        connect(&con_str, NoTls)
            .unwrap()
    }


    #[test]
    fn tests() {
        let node = run_postgres();
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
        let k = account_type_dao.get_all_account_types_for_tenant_id(&1);
        assert!(k.len() > 5);
    }
}
