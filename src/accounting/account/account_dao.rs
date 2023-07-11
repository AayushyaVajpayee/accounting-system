use std::sync::OnceLock;

use postgres::Client;

use crate::accounting::account::account_models::{Account, CreateAccountRequest};
use crate::accounting::currency::currency_models::AuditMetadataBase;

pub trait AccountDao {
    fn get_account_by_id(&mut self, id: &i32) -> Option<Account>;
    fn create_account(&mut self, request: &CreateAccountRequest) -> i32;
}

pub struct AccountDaoPostgresImpl {
    postgres_client: Client,
}

const ACCOUNT_POSTGRES_SELECT_FIELDS: &str = "id,tenant_id,display_code,account_type_id,user_id,ledger_master_id,debits_posted,debits_pending,credits_posted,credits_pending,created_by,updated_by,created_at,updated_at";
const ACCOUNT_TABLE_NAME: &str = "user_account";
static ACCOUNT_BY_ID_QUERY: OnceLock<String> = OnceLock::new();
static ACCOUNT_INSERT_STATEMENT: OnceLock<String> = OnceLock::new();

impl AccountDaoPostgresImpl {
    fn get_account_by_id_query() -> &'static str {
        ACCOUNT_BY_ID_QUERY.get_or_init(|| {
            format!("select {ACCOUNT_POSTGRES_SELECT_FIELDS} from {ACCOUNT_TABLE_NAME} where id=$1")
        })
    }

    fn get_insert_statement() -> &'static str {
        ACCOUNT_INSERT_STATEMENT.get_or_init(|| {
            format!("insert into {ACCOUNT_TABLE_NAME} ({ACCOUNT_POSTGRES_SELECT_FIELDS}) values\
            (DEFAULT,$1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) returning id")
        })
    }
}

impl AccountDao for AccountDaoPostgresImpl {
    fn get_account_by_id(&mut self, id: &i32) -> Option<Account> {
        let query = AccountDaoPostgresImpl::get_account_by_id_query();
        let k = self.postgres_client.query(
            query,
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
        let query = AccountDaoPostgresImpl::get_insert_statement();
        self.postgres_client.query(
            query,
            &[
                &request.tenant_id,
                &request.display_code,
                &request.account_type_id,
                &request.user_id,
                &request.ledger_master_id,
                &0i64, &0i64, &0i64, &0i64,
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
mod account_tests {
    use postgres::{Client, NoTls};

    use crate::accounting::account::account_dao::{AccountDao, AccountDaoPostgresImpl};
    use crate::accounting::account::account_models::{a_create_account_request, CreateAccountRequestTestBuilder};
    use crate::seeddata::seed_service::copy_tables;
    use crate::test_utils::test_utils_postgres::run_postgres;

    fn create_postgres_client(port: u16) -> Client {
        let con_str =
            format!("host=localhost user=postgres password=postgres dbname=postgres port={port}");
        let client = Client::
        connect(&con_str, NoTls)
            .unwrap();
        client
    }

    #[test]
    fn test_account() {
        let node = run_postgres();
        let port = node.get_host_port_ipv4(5432);
        let mut postgres_client = create_postgres_client(port);
        copy_tables(port);
        let an_account_request = a_create_account_request(CreateAccountRequestTestBuilder {
            tenant_id: Some(1),
            ledger_master_id: Some(1),
            account_type_id: Some(1),
            user_id: Some(1),
            ..Default::default()
        });
        let mut account_dao = AccountDaoPostgresImpl { postgres_client: postgres_client };
        let account_id = account_dao.create_account(&an_account_request);
        let account_fetched = account_dao.get_account_by_id(&account_id).unwrap();
    }
}