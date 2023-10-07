use std::sync::Arc;

use async_trait::async_trait;
use moka::future::Cache;

use crate::accounting::postgres_factory::get_postgres_conn_pool;
use crate::masters::pincode_master::pincode_master_dao::{get_pincode_master_dao, PincodeMasterDao};
use crate::masters::pincode_master::pincode_models::PincodeMaster;

const CACHE_ALL_KEY: i32 = 1;
#[async_trait]
pub trait PincodeMasterService {
    async fn get_all_pincodes(&self)->Option<Arc<Vec<Arc<PincodeMaster>>>>;
    async fn get_pincode_by_id(&self, id: i32)->Option<Arc<PincodeMaster>>;
}
#[allow(dead_code)]
pub async fn get_pincode_master_service() -> Box<dyn PincodeMasterService + Send + Sync> {
    let pclient = get_postgres_conn_pool();
    let city_master_dao = get_pincode_master_dao(pclient);
    let cache: Cache<i32, Arc<Vec<Arc<PincodeMaster>>>> = Cache::new(1);
    let city_master_service = PincodeMasterServiceImpl {
        dao: city_master_dao,
        cache_all: cache,
        cache_by_id: Cache::new(25000),
    };
    Box::new(city_master_service)
}


struct PincodeMasterServiceImpl{
    dao:Box<dyn PincodeMasterDao +Send+Sync>,
    cache_all: Cache<i32,Arc<Vec<Arc<PincodeMaster>>>>,
    cache_by_id:Cache<i32,Arc<PincodeMaster>>
}

impl PincodeMasterServiceImpl{
    async fn populate_caches(&self) {
        let cache = self.cache_all.clone();
        let cache_by_id = self.cache_by_id.clone();
        let db_pincode_list = self.dao.get_all_pincodes().await;
        let mut cache_vec: Vec<Arc<PincodeMaster>> = Vec::with_capacity(db_pincode_list.len());
        for pincode in db_pincode_list.into_iter() {
            let arc_city = Arc::new(pincode);
            cache_vec.push(arc_city.clone());
            cache_by_id.insert(arc_city.id, arc_city.clone()).await;
        }
        cache.insert(CACHE_ALL_KEY, Arc::new(cache_vec)).await;
    }
}
#[async_trait]
impl PincodeMasterService for PincodeMasterServiceImpl{
    async fn get_all_pincodes(&self) -> Option<Arc<Vec<Arc<PincodeMaster>>>> {
        let cache = self.cache_all.clone();
        let res = cache.get(&CACHE_ALL_KEY).await;
        if res.is_none() {
            self.populate_caches().await;
            return cache.get(&CACHE_ALL_KEY).await;
        }
        return res;
    }

    async fn get_pincode_by_id(&self, id: i32) -> Option<Arc<PincodeMaster>> {
        if self.cache_all.get(&CACHE_ALL_KEY).await.is_none() {
            self.populate_caches().await;
        }
        return self.cache_by_id.get(&id).await;
    }
}


#[cfg(test)]
mod tests{
    use moka::future::Cache;
    use spectral::assert_that;
    use spectral::option::OptionAssertions;
    use crate::masters::country_master::country_model::INDIA_COUNTRY_ID;

    use crate::masters::pincode_master::pincode_master_dao::MockPincodeMasterDao;
    use crate::masters::pincode_master::pincode_master_service::{PincodeMasterService, PincodeMasterServiceImpl};
    use crate::masters::pincode_master::pincode_models::{Pincode, PincodeMaster};

    #[tokio::test]
    async fn test_get_all_pincodes_to_be_called_once_and_then_entry_to_be_fetched_from_cache(){
        let mut dao_mock = MockPincodeMasterDao::new();
        dao_mock.expect_get_all_pincodes()
            .times(1)
            .returning(||{
                vec![PincodeMaster{
                    id: 0,
                    pincode: Pincode::new("123456",*INDIA_COUNTRY_ID).unwrap(),
                    city_id:3,
                    audit_metadata: Default::default(),
                    country_id: *INDIA_COUNTRY_ID
                }]
            });
        let service = PincodeMasterServiceImpl{
            dao:Box::new(dao_mock),
            cache_all:Cache::new(1),
            cache_by_id:Cache::new(25000)
        };
        let p = service.get_all_pincodes().await.unwrap();
        let p1 = service.get_all_pincodes().await.unwrap();
        assert_eq!(p.len(),1);
        assert_eq!(p1.len(),1);
    }


    #[tokio::test]
    async fn test_get_pincode_by_id() {
        let mut dao_mock = MockPincodeMasterDao::new();
        dao_mock.expect_get_all_pincodes()
            .times(1)
            .returning(||{
                vec![PincodeMaster{
                    id: 0,
                    pincode: Pincode::new("123456", *INDIA_COUNTRY_ID).unwrap(),
                    city_id: 0,
                    audit_metadata: Default::default(),
                    country_id:*INDIA_COUNTRY_ID
                }]
            });
        let service = PincodeMasterServiceImpl{
            dao: Box::new(dao_mock),
            cache_all: Cache::new(1),
            cache_by_id: Cache::new(25000),
        };

        let p = service.get_pincode_by_id(0).await;
        let p1 = service.get_pincode_by_id(0).await;
        assert_that!(p).is_some();
        assert_that!(p1).is_some();
    }
}