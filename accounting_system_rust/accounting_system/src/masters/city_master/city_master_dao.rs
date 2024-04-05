use std::sync::Arc;

use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::Pool;
#[cfg(test)]
use mockall::automock;
use tokio_postgres::Row;
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::city_master::city_master_models::{CityMaster, CityName};

const SELECT_FIELDS: &str =
    "id,city_name,state_id,created_by,updated_by,created_at,updated_at,country_id";

const TABLE_NAME: &str = "city_master";

const FETCH_ALL_QUERY: &str = concatcp!("select ", SELECT_FIELDS, " from ", TABLE_NAME);

const BY_ID_QUERY: &str = concatcp!(
    "select ",
    SELECT_FIELDS,
    " from ",
    TABLE_NAME,
    " where id=$1"
);

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CityMasterDao: Send + Sync {
    async fn get_all_cities(&self) -> Vec<CityMaster>;

    async fn get_city_by_id(&self, id: &Uuid) -> Option<CityMaster>;
}
#[allow(dead_code)]
pub fn get_city_master_dao(client: Arc<Pool>) -> Arc<dyn CityMasterDao> {
    let city_master_dao = CityMasterDaoImpl {
        postgres_client: client,
    };
    Arc::new(city_master_dao)
}
struct CityMasterDaoImpl {
    postgres_client: Arc<Pool>,
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
            country_id: row.get(7),
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

    async fn get_city_by_id(&self, id: &Uuid) -> Option<CityMaster> {
        let conn = self.postgres_client.get().await.unwrap();
        let rows = conn.query(BY_ID_QUERY, &[&id]).await.unwrap();
        rows.iter().map(|row| row.try_into().unwrap()).next()
    }
}

#[cfg(test)]
mod tests {
    use speculoos::assert_that;
    use speculoos::option::OptionAssertions;

    use crate::accounting::postgres_factory::test_utils_postgres::{
        get_dao_generic, get_postgres_conn_pool, get_postgres_image_port,
    };
    use crate::masters::city_master::city_master_dao::{CityMasterDao, CityMasterDaoImpl};
    use crate::masters::city_master::city_master_models::tests::SEED_CITY_ID;

    #[tokio::test]
    async fn should_be_able_to_fetch_all_cities() {
        let city_master_dao = get_dao_generic(
            |a| CityMasterDaoImpl {
                postgres_client: a.clone(),
            },
            None,
        )
        .await;
        let cities = city_master_dao.get_all_cities().await;
        assert!(!cities.is_empty());
    }

    #[tokio::test]
    async fn should_be_able_to_fetch_city_by_id() {
        let city_master_dao = get_dao_generic(
            |a| CityMasterDaoImpl {
                postgres_client: a.clone(),
            },
            None,
        )
        .await;
        let city = city_master_dao.get_city_by_id(&SEED_CITY_ID).await;
        assert_that!(city).is_some();
    }
}
