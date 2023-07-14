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
    use postgres::{Client, NoTls};
    use testcontainers::clients;
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;

    use crate::accounting::currency::currency_dao::{CurrencyDao, CurrencyDaoPostgresImpl};
    use crate::accounting::currency::currency_models::{a_create_currency_master_request, CreateCurrencyMasterRequestTestBuilder};
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
        copy_tables(port);
        let currency_master = a_create_currency_master_request(
            CreateCurrencyMasterRequestTestBuilder {
                tenant_id: Some(1),
                ..Default::default()
            }
        );
        let mut currency_dao = CurrencyDaoPostgresImpl { postgres_client: postgres_client };
        currency_dao.create_currency_entry(&currency_master);
        let got_c = currency_dao.get_currency_entry_by_id(&1);
        println!("{:?}", got_c)
    }
}