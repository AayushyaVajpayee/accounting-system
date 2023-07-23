use std::sync::OnceLock;

use postgres::{Client, Row};

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::ledger::ledgermaster::ledger_master_models::{CreateLedgerMasterEntryRequest, LedgerMaster};

pub trait LedgerMasterDao {
    fn get_ledger_master_by_id(&mut self, id: &i32) -> Option<LedgerMaster>;
    fn create_ledger_master_entry(&mut self, request: &CreateLedgerMasterEntryRequest) -> i32;
}


pub fn get_ledger_master_dao(client: Client) -> Box<dyn LedgerMasterDao> {
    let p = LedgerMasterPostgresDaoImpl {
        postgres_client: client
    };
    Box::new(p)
}


struct LedgerMasterPostgresDaoImpl {
    postgres_client: Client,
}

const SELECT_FIELDS: &str = "id,tenant_id,display_name,\
currency_master_id,created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "ledger_master";
static BY_ID_QUERY: OnceLock<String> = OnceLock::new();
static INSERT_STATEMENT: OnceLock<String> = OnceLock::new();

impl TryFrom<&Row> for LedgerMaster {
    type Error = ();

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(LedgerMaster {
            id: row.get(0),
            tenant_id: row.get(1),
            display_name: row.get(2),
            currency_master_id: row.get(3),
            audit_metadata: AuditMetadataBase {
                created_by: row.get(4),
                updated_by: row.get(5),
                created_at: row.get(6),
                updated_at: row.get(7),
            },
        })
    }
}

impl LedgerMasterPostgresDaoImpl {
    fn get_ledger_master_entry_by_id_query() -> &'static String {
        BY_ID_QUERY.get_or_init(|| {
            format!("select {} from {} where id=$1", SELECT_FIELDS, TABLE_NAME)
        })
    }
    fn create_insert_statement() -> &'static String {
        INSERT_STATEMENT.get_or_init(|| {
            format!("insert into {} ({}) values\
             (DEFAULT,$1,$2,$3,$4,$5,$6,$7) returning id",
                    TABLE_NAME,
                    SELECT_FIELDS)
        })
    }
}

impl LedgerMasterDao for LedgerMasterPostgresDaoImpl {
    fn get_ledger_master_by_id(&mut self, id: &i32) -> Option<LedgerMaster> {
        let query = LedgerMasterPostgresDaoImpl::get_ledger_master_entry_by_id_query();
        let values = self.postgres_client.query(query, &[id]).unwrap();
        values.iter()
            .map(|row|
                row.try_into()
                    .unwrap())
            .next()
    }

    fn create_ledger_master_entry(&mut self, request: &CreateLedgerMasterEntryRequest) -> i32 {
        let query = LedgerMasterPostgresDaoImpl::create_insert_statement();
        self.postgres_client.query(query,
                                   &[
                                       &request.tenant_id,
                                       &request.display_name,
                                       &request.currency_master_id,
                                       &request.audit_metadata.created_by,
                                       &request.audit_metadata.updated_by,
                                       &request.audit_metadata.created_at,
                                       &request.audit_metadata.updated_at,
                                   ]).unwrap()
            .iter()
            .map(|row| row.get(0))
            .next()
            .unwrap()
    }
}


#[cfg(test)]
mod tests {
    use crate::ledger::ledgermaster::ledger_master_dao::{LedgerMasterDao, LedgerMasterPostgresDaoImpl};
    use crate::ledger::ledgermaster::ledger_master_models::a_create_ledger_master_entry_request;
    use crate::test_utils::test_utils_postgres::{create_postgres_client, get_postgres_image_port};

    #[test]
    fn test_create_and_insert() {
        let port = get_postgres_image_port();
        let postgres_client = create_postgres_client(port);
        let ledger_master = a_create_ledger_master_entry_request(
            Default::default());
        let mut ledger_master_dao = LedgerMasterPostgresDaoImpl { postgres_client };
        let id = ledger_master_dao.create_ledger_master_entry(&ledger_master);
        let _queried = ledger_master_dao.get_ledger_master_by_id(&id).unwrap();
    }
}