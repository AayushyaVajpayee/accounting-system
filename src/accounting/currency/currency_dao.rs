use std::sync::OnceLock;

use postgres::{Client, Row};

use crate::accounting::currency::currency_models::{AuditMetadataBase, CreateCurrencyMasterRequest, CurrencyMaster};

const SELECT_FIELDS: &str = "id,tenant_id,scale,display_name,description,\
created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "currency_master";
static BY_ID_QUERY: OnceLock<String> = OnceLock::new();
static INSERT_STATEMENT: OnceLock<String> = OnceLock::new();

pub trait CurrencyDao {
    fn get_currency_entry_by_id(&mut self, id: &i16) -> Option<CurrencyMaster>;
    fn create_currency_entry(&mut self, currency: &CreateCurrencyMasterRequest) -> i16;
}

pub struct CurrencyDaoPostgresImpl {
    postgres_client: Client,
}

impl TryFrom<&Row> for CurrencyMaster {
    type Error = ();

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(CurrencyMaster {
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
        })
    }
}

impl CurrencyDaoPostgresImpl {
    fn get_currency_master_by_id_query() -> &'static String {
        BY_ID_QUERY.get_or_init(|| {
            format!("select {} from {} where id=$1", SELECT_FIELDS, TABLE_NAME)
        })
    }
    fn create_insert_statement() -> &'static String {
        INSERT_STATEMENT.get_or_init(|| {
            format!("insert into {} ({}) values \
            (DEFAULT,$1,$2,$3,$4,$5,$6,$7,$8) returning id",
                    TABLE_NAME,
                    SELECT_FIELDS)
        })
    }
}

pub fn get_currency_dao(client: Client) -> Box<dyn CurrencyDao> {
    let currency_dao = CurrencyDaoPostgresImpl {
        postgres_client: client
    };
    Box::new(currency_dao)
}

impl CurrencyDao for CurrencyDaoPostgresImpl {
    fn get_currency_entry_by_id(&mut self, id: &i16) -> Option<CurrencyMaster> {
        let query = CurrencyDaoPostgresImpl::get_currency_master_by_id_query();
        let k = self.postgres_client.
            query(query, &[id]).unwrap();
        k.iter().map(|row|
            row.try_into().unwrap()).next()
    }

    fn create_currency_entry(&mut self, currency: &CreateCurrencyMasterRequest) -> i16 {
        let query = CurrencyDaoPostgresImpl::create_insert_statement();
        self.postgres_client.query(
            query,
            &[&(currency.tenant_id), &(currency.scale), &currency.display_name,
                &currency.description,
                &currency.audit_metadata.created_by, &currency.audit_metadata.updated_by,
                &(currency.audit_metadata.created_at), &(currency.audit_metadata.updated_at)],
        ).unwrap().iter().map(|row| row.get(0)).next().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::accounting::currency::currency_dao::{CurrencyDao, CurrencyDaoPostgresImpl};
    use crate::accounting::currency::currency_models::{a_create_currency_master_request, CreateCurrencyMasterRequestTestBuilder};
    use crate::test_utils::test_utils_postgres::{create_postgres_client, get_postgres_image_port};

    #[test]
    fn should_be_able_to_create_and_fetch_currency() {
        let port = get_postgres_image_port();
        let postgres_client = create_postgres_client(port);
        let currency_master = a_create_currency_master_request(
            CreateCurrencyMasterRequestTestBuilder {
                tenant_id: Some(1),
                ..Default::default()
            }
        );
        let mut currency_dao = CurrencyDaoPostgresImpl { postgres_client: postgres_client };
        let curr_id = currency_dao.create_currency_entry(&currency_master);
        let fetched_curr = currency_dao.get_currency_entry_by_id(&curr_id).unwrap();
        assert_eq!(curr_id, fetched_curr.id)
    }
}