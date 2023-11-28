use std::sync::Arc;

use async_trait::async_trait;
use moka::future::Cache;
use uuid::Uuid;

use crate::accounting::postgres_factory::get_postgres_conn_pool;
use crate::masters::city_master::city_master_dao::{CityMasterDao, get_city_master_dao};
use crate::masters::city_master::city_master_models::CityMaster;

const CACHE_ALL_KEY: i32 = 1;

#[async_trait]
pub trait CityMasterService:Send+Sync {
    async fn get_city_by_id(&self, id: &Uuid) -> Option<Arc<CityMaster>>;
    async fn get_all_cities(&self) -> Option<Arc<Vec<Arc<CityMaster>>>>;
}

struct CityMasterServiceImpl {
    dao: Arc<dyn CityMasterDao>,
    cache_all: Cache<i32, Arc<Vec<Arc<CityMaster>>>>,
    cache_by_id: Cache<Uuid, Arc<CityMaster>>,
}

pub fn get_city_master_service() -> Arc<dyn CityMasterService> {
    let pclient = get_postgres_conn_pool();
    let city_master_dao = get_city_master_dao(pclient);
    let cache: Cache<i32, Arc<Vec<Arc<CityMaster>>>> = Cache::new(733);
    let city_master_service = CityMasterServiceImpl {
        dao: city_master_dao,
        cache_all: cache,
        cache_by_id: Cache::new(733),
    };
    Arc::new(city_master_service)
}
impl CityMasterServiceImpl {
    async fn populate_caches(&self) {
        let cache = self.cache_all.clone();
        let cache_by_id = self.cache_by_id.clone();
        let db_city_list = self.dao.get_all_cities().await;
        let mut cache_vec: Vec<Arc<CityMaster>> = Vec::with_capacity(db_city_list.len());
        for city in db_city_list.into_iter() {
            let arc_city = Arc::new(city);
            cache_vec.push(arc_city.clone());
            cache_by_id.insert(arc_city.id, arc_city.clone()).await;
        }
        cache.insert(CACHE_ALL_KEY, Arc::new(cache_vec)).await;
    }
}
#[async_trait]
impl CityMasterService for CityMasterServiceImpl {
    async fn get_city_by_id(&self, id: &Uuid) -> Option<Arc<CityMaster>> {
        if self.cache_all.get(&CACHE_ALL_KEY).await.is_none() {
            self.populate_caches().await;
        }
        return self.cache_by_id.get(&id).await;
    }

    async fn get_all_cities(&self) -> Option<Arc<Vec<Arc<CityMaster>>>> {
        let cache = self.cache_all.clone();
        let res = cache.get(&CACHE_ALL_KEY).await;
        if res.is_none() {
            self.populate_caches().await;
            return cache.get(&CACHE_ALL_KEY).await;
        }
        return res;
    }
}

#[cfg(test)]
mod tests{
    use std::sync::Arc;
    use moka::future::Cache;
    use spectral::assert_that;
    use spectral::option::OptionAssertions;
    use uuid::Uuid;

    use crate::masters::city_master::city_master_dao::MockCityMasterDao;
    use crate::masters::city_master::city_master_models::{CityMaster, CityName};
    use crate::masters::city_master::city_master_service::{CityMasterService, CityMasterServiceImpl};
    use crate::masters::state_master::state_models::tests::SEED_STATE_ID;

    #[tokio::test]
   async fn test_get_all_cities_to_be_called_once_and_then_entry_to_be_fetched_from_cache(){
        let mut dao_mock = MockCityMasterDao::new();
        dao_mock.expect_get_all_cities()
            .times(1)
            .returning(||{
                vec![CityMaster{
                    id: Default::default(),
                    city_name: CityName::new("Haridwar").unwrap(),
                    state_id: *SEED_STATE_ID,
                    audit_metadata: Default::default(),
                    country_id:Uuid::now_v7()
                }]
            });
        let service = CityMasterServiceImpl{
            dao:Arc::new(dao_mock),
            cache_all:Cache::new(1),
            cache_by_id:Cache::new(750)
        };
        let p = service.get_all_cities().await.unwrap();
        let p1 = service.get_all_cities().await.unwrap();
        assert_eq!(p.len(),1);
        assert_eq!(p1.len(),1);
    }
    #[tokio::test]
    async fn test_get_city_by_id() {
        let mut dao_mock = MockCityMasterDao::new();
        dao_mock.expect_get_all_cities()
            .times(1)
            .returning(||{
                vec![CityMaster{
                    id: Default::default(),
                    city_name: CityName::new("Haridwar").unwrap(),
                    state_id: *SEED_STATE_ID,
                    audit_metadata: Default::default(),
                    country_id:Uuid::now_v7()
                }]
            });
        let service = CityMasterServiceImpl{
            dao: Arc::new(dao_mock),
            cache_all: Cache::new(1),
            cache_by_id: Cache::new(740),
        };
        let id = Uuid::default();
        let p = service.get_city_by_id(&id).await;
        let p1 = service.get_city_by_id(&id).await;
        assert_that!(p).is_some();
        assert_that!(p1).is_some();
    }
}