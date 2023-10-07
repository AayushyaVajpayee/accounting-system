use crate::accounting::postgres_factory::get_postgres_conn_pool;
use crate::masters::country_master::country_dao::{get_country_master_dao, CountryMasterDao};
use crate::masters::country_master::country_model::{CountryEnum, CountryMaster, INDIA_COUNTRY_ID};
use async_trait::async_trait;
use moka::future::Cache;
use std::sync::Arc;
use uuid::Uuid;

const CACHE_ALL_KEY: i32 = 1;

#[async_trait]
pub trait CountryMasterService {
    async fn get_all_countries(&self) -> Option<Arc<Vec<Arc<CountryMaster>>>>;
    async fn get_country_by_id(&self, id: Uuid) -> Option<Arc<CountryMaster>>;
}

struct CountryMasterServiceImpl {
    dao: Box<dyn CountryMasterDao + Send + Sync>,
    cache_by_id: Cache<Uuid, Arc<CountryMaster>>,
    cache_all: Cache<i32, Arc<Vec<Arc<CountryMaster>>>>,
}

#[allow(dead_code)]
pub async fn get_country_master_service() -> Box<dyn CountryMasterService + Send + Sync> {
    let pclient = get_postgres_conn_pool();
    let country_master_dao = get_country_master_dao(pclient);
    let country_master_service = CountryMasterServiceImpl {
        dao: country_master_dao,
        cache_by_id: Cache::new(200),
        cache_all: Cache::new(1),
    };
    Box::new(country_master_service)
}

impl CountryMasterServiceImpl {
    async fn populate_caches(&self) {
        let cache = self.cache_all.clone();
        let cache_by_id = self.cache_by_id.clone();
        let db_state_list = self.dao.get_all_countries().await;
        let mut cache_vec: Vec<Arc<CountryMaster>> = Vec::with_capacity(db_state_list.len());
        for country in db_state_list.into_iter() {
            let arc_country = Arc::new(country);
            cache_vec.push(arc_country.clone());
            cache_by_id
                .insert(arc_country.id, arc_country.clone())
                .await
        }
        cache.insert(CACHE_ALL_KEY,Arc::new(cache_vec)).await;
    }
}

#[async_trait]
impl CountryMasterService for CountryMasterServiceImpl {
    async fn get_all_countries(&self) -> Option<Arc<Vec<Arc<CountryMaster>>>> {
        let cache = self.cache_all.clone();
        let res = cache.get(&CACHE_ALL_KEY).await;
        if res.is_none(){
            self.populate_caches().await;
            return cache.get(&CACHE_ALL_KEY).await;
        }
        return res
    }

    async fn get_country_by_id(&self, id: Uuid) -> Option<Arc<CountryMaster>> {
        if self.cache_all.get(&CACHE_ALL_KEY).await.is_none() {
            self.populate_caches().await;
        }
        let cache = self.cache_by_id.clone();
        let item = cache.get(&id).await;
        return item;
    }

}

#[cfg(test)]
mod tests{
    use moka::future::Cache;
    use spectral::assert_that;
    use spectral::option::OptionAssertions;
    use uuid::Uuid;
    use crate::masters::country_master::country_dao::MockCountryMasterDao;
    use crate::masters::country_master::country_model::{CountryMaster, CountryName};
    use crate::masters::country_master::country_service::{CountryMasterService, CountryMasterServiceImpl};
    #[tokio::test]
    async fn test_get_all_countries_should_be_called_once_and_then_entry_to_be_fetched_from_cache(){
        let mut dao_mock = MockCountryMasterDao::new();
        dao_mock.expect_get_all_countries().times(1).returning(|| {
            vec![CountryMaster {
                id:Uuid::now_v7() ,
                name: CountryName::new("INDIA").unwrap(),
                audit_metadata: Default::default(),
            }]
        });
        let service = CountryMasterServiceImpl {
            dao: Box::new(dao_mock),
            cache_all: Cache::new(1),
            cache_by_id: Cache::new(200),
        };
        let p = service.get_all_countries().await.unwrap();
        let _p1 = service.get_all_countries().await;
        assert_eq!(p.len(), 1);
    }

    #[tokio::test]
    async fn test_get_country_by_id(){
        let mut dao_mock = MockCountryMasterDao::new();
        let id=Uuid::now_v7();
        dao_mock.expect_get_all_countries().times(1).returning(move ||{
            vec![CountryMaster {
                id,
                name: CountryName::new("INDIA").unwrap(),
                audit_metadata: Default::default(),
            }]
        });
        let service = CountryMasterServiceImpl{
            dao: Box::new(dao_mock),
            cache_all: Cache::new(1),
            cache_by_id: Cache::new(200),
        };
        let p = service.get_country_by_id(id).await;
        let p1 = service.get_country_by_id(id).await;
        assert_that!(p).is_some();
        assert_that!(p1).is_some();

    }
}