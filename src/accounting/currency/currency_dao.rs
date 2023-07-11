use std::error::Error;

use postgres::Client;
use postgres::types::ToSql;

use crate::accounting::currency::currency_models::{AuditMetadataBase, CreateCurrencyMasterRequest, CurrencyMaster};

pub trait CurrencyDao {
    fn get_currency_entry_by_id(&mut self, id: &i16) -> Option<CurrencyMaster>;
    fn create_currency_entry(&mut self, currency: &CreateCurrencyMasterRequest) -> i16;
}

pub struct CurrencyDaoPostgresImpl {
    postgres_client: Client,
}

pub fn get_currency_dao(client: Client) -> Box<dyn CurrencyDao> {
    let currency_dao = CurrencyDaoPostgresImpl {
        postgres_client: client
    };
    Box::new(currency_dao)
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

    fn create_currency_entry(&mut self, currency: &CreateCurrencyMasterRequest) -> i16 {
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
}

#[cfg(test)]
mod tests {
    use postgres::{Client, NoTls};

    use crate::accounting::currency::currency_dao::{CurrencyDao, CurrencyDaoPostgresImpl};
    use crate::accounting::currency::currency_models::{a_create_currency_master_request, CreateCurrencyMasterRequestTestBuilder};
    use crate::test_utils::test_utils_postgres::get_postgres_image_port;

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
        let port = get_postgres_image_port();
        let mut postgres_client = create_postgres_client(port);
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