use std::sync::Arc;
use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::country_master::country_model::{CountryMaster, CountryName};
use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::{GenericClient, Pool};
#[cfg(test)]
use mockall::automock;
use tokio_postgres::Row;

const SELECT_FIELDS: &str = "id,name,created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "country_master";

const FETCH_ALL_QUERY: &str = concatcp!("select ", SELECT_FIELDS, " from ", TABLE_NAME);

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CountryMasterDao:Send+Sync {
    async fn get_all_countries(&self) -> Vec<CountryMaster>;
}

struct CountryMasterDaoPostgresImpl {
    postgres_client: &'static Pool,
}

impl TryFrom<&Row> for CountryMaster {
    type Error = &'static str;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(CountryMaster {
            id: row.get(0),
            name: CountryName::new(row.get(1))?,
            audit_metadata: AuditMetadataBase {
                created_by: row.get(2),
                updated_by: row.get(3),
                created_at: row.get(4),
                updated_at: row.get(5),
            },
        })
    }
}

pub fn get_country_master_dao(client: &'static Pool)->Arc<dyn CountryMasterDao>{
    let country_master_dao = CountryMasterDaoPostgresImpl{
        postgres_client:client
    };
    Arc::new(country_master_dao)
}

#[async_trait]
impl CountryMasterDao for CountryMasterDaoPostgresImpl{
    async fn get_all_countries(&self) -> Vec<CountryMaster> {
        let conn = self.postgres_client.get().await.unwrap();
        let rows = conn.query(FETCH_ALL_QUERY,&[]).await.unwrap();
        rows.iter().map(|row|row.try_into().unwrap()).collect()
    }
}

#[cfg(test)]
mod tests{
    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::masters::country_master::country_dao::{CountryMasterDao, CountryMasterDaoPostgresImpl};

    #[tokio::test]
    async fn should_be_able_to_fetch_all_countries(){
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let country_master_dao = CountryMasterDaoPostgresImpl{
            postgres_client
        };
        let p = country_master_dao.get_all_countries().await;
        assert!(!p.is_empty())
    }
}