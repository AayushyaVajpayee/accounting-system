use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::city_master::city_master_models::{CityMaster, CityName};
use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::Pool;
use futures_util::TryFutureExt;
#[cfg(test)]
use mockall::automock;
use tokio_postgres::Row;

const SELECT_FIELDS: &str = "id,city_name,state_id,created_by,updated_by,created_at,updated_at";

const TABLE_NAME: &str = "city_master";

const FETCH_ALL_QUERY: &str = concatcp!("select ", SELECT_FIELDS, " from ", TABLE_NAME);

const BY_ID_QUERY: &str = concatcp!(
    "select ",
    SELECT_FIELDS,
    " from ",
    TABLE_NAME,
    " where id=$1"
);


pub fn get_city_master_dao(client:&'static Pool)->Box<dyn CityMasterDao +Send+Sync>{
    let city_master_dao = CityMasterDaoImpl{
        postgres_client:client
    };
    Box::new(city_master_dao)
}
#[cfg_attr(test, automock)]
#[async_trait]
pub trait CityMasterDao {
    async fn get_all_cities(&self) -> Vec<CityMaster>;

    async fn get_city_by_id(&self, id: i32) -> Option<CityMaster>;
}

struct CityMasterDaoImpl {
    postgres_client: &'static Pool,
}

impl TryFrom<&Row> for CityMaster {
    type Error = &'static str;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(CityMaster {
            id: row.get(0),
            city_name: CityName::new(row.get(1))?,
            state_id: row.get(2),
            audit_metadata: AuditMetadataBase {
                created_by: row.get(3),
                updated_by: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
            },
        })
    }
}

#[async_trait]
impl CityMasterDao for CityMasterDaoImpl {
    async fn get_all_cities(&self) -> Vec<CityMaster> {
        let conn = self.postgres_client.get().await.unwrap();
        let rows = conn.query(FETCH_ALL_QUERY, &[]).await.unwrap();
        rows.iter().map(|row| row.try_into().unwrap()).collect()
    }

    async fn get_city_by_id(&self, id: i32) -> Option<CityMaster> {
        let conn = self.postgres_client.get().await.unwrap();
        let rows = conn.query(BY_ID_QUERY, &[&id]).await.unwrap();
        rows.iter().map(|row| row.try_into().unwrap()).next()
    }
}


#[cfg(test)]
mod tests{
    use spectral::assert_that;
    use spectral::option::OptionAssertions;
    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::masters::city_master::city_master_dao::{CityMasterDao, CityMasterDaoImpl};

    #[tokio::test]
    async fn should_be_able_to_fetch_all_cities(){
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let city_master_dao = CityMasterDaoImpl{postgres_client};
        let cities = city_master_dao.get_all_cities().await;
        assert!(!cities.is_empty());
    }

    #[tokio::test]
    async fn should_be_able_to_fetch_city_by_id(){
        let port =get_postgres_image_port().await;
        let postgres_client =  get_postgres_conn_pool(port).await;
        let city_master_dao = CityMasterDaoImpl{postgres_client};
        let city =  city_master_dao.get_city_by_id(0).await;
        assert_that!(city).is_some();
    }
}