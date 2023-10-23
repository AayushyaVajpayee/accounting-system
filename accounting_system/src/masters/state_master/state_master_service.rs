use std::sync::Arc;

use async_trait::async_trait;
use moka::future::Cache;

use crate::accounting::postgres_factory::get_postgres_conn_pool;
use crate::masters::state_master::state_master_dao::{get_state_master_dao, StateMasterDao};
use crate::masters::state_master::state_models::StateMasterModel;

const CACHE_ALL_KEY: i32 = 1;
#[async_trait]
pub trait StateMasterService:Send+Sync {
    async fn get_all_states(&self) -> Option<Arc<Vec<Arc<StateMasterModel>>>>;
    async fn get_state_by_id(&self, id: i32) -> Option<Arc<StateMasterModel>>;
}

struct StateMasterServiceImpl {
    dao: Arc<dyn StateMasterDao>,
    cache_by_id: Cache<i32, Arc<StateMasterModel>>,
    cache_all: Cache<i32, Arc<Vec<Arc<StateMasterModel>>>>,
}
pub fn get_state_master_service() -> Arc<dyn StateMasterService> {
    let pclient = get_postgres_conn_pool();
    let state_master_dao = get_state_master_dao(pclient);
    let cache: Cache<i32, Arc<Vec<Arc<StateMasterModel>>>> = Cache::new(40);
    let state_master_s = StateMasterServiceImpl {
        dao: state_master_dao,
        cache_by_id: Cache::new(40),
        cache_all: cache,
    };
    Arc::new(state_master_s)
}

impl StateMasterServiceImpl {
    async fn populate_caches(&self) {
        let cache = self.cache_all.clone();
        let cache_by_id = self.cache_by_id.clone();
        let db_state_list = self.dao.get_all_states().await;
        let mut cache_vec: Vec<Arc<StateMasterModel>> = Vec::with_capacity(db_state_list.len());
        for state in db_state_list.into_iter() {
            let arc_state = Arc::new(state);
            cache_vec.push(arc_state.clone());
            cache_by_id.insert(arc_state.id, arc_state.clone()).await;
        }
        cache.insert(CACHE_ALL_KEY, Arc::new(cache_vec)).await;
    }
}

#[async_trait]
impl StateMasterService for StateMasterServiceImpl {
    async fn get_all_states(&self) -> Option<Arc<Vec<Arc<StateMasterModel>>>> {
        let cache = self.cache_all.clone();
        let res = cache.get(&CACHE_ALL_KEY).await;
        if res.is_none() {
            self.populate_caches().await;
            return cache.get(&CACHE_ALL_KEY).await;
        }
        return res //safe to call unwrap because we are initialising with empty vector
    }

    async fn get_state_by_id(&self, id: i32) -> Option<Arc<StateMasterModel>> {
        if self.cache_all.get(&CACHE_ALL_KEY).await.is_none() {
            self.populate_caches().await;
        }
        let cache = self.cache_by_id.clone();
        let item = cache.get(&id).await;
        return item;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use moka::future::Cache;
    use spectral::assert_that;
    use spectral::prelude::OptionAssertions;
    use uuid::Uuid;

    use crate::masters::state_master::state_master_dao::MockStateMasterDao;
    use crate::masters::state_master::state_master_service::{
        StateMasterService, StateMasterServiceImpl,
    };
    use crate::masters::state_master::state_models::{StateMasterModel, StateName};

    #[tokio::test]
    async fn test_get_all_states_should_be_called_once_and_then_entry_to_be_fetched_from_cache() {
        let mut dao_mock = MockStateMasterDao::new();
        dao_mock.expect_get_all_states().times(1).returning(|| {
            vec![StateMasterModel {
                id: 0,
                state_name: StateName::new("Uttarakhand").unwrap(),
                audit_metadata: Default::default(),
                country_id:Uuid::now_v7()
            }]
        });
        let service = StateMasterServiceImpl {
            dao: Arc::new(dao_mock),
            cache_all: Cache::new(1),
            cache_by_id: Cache::new(40),
        };
        let p = service.get_all_states().await.unwrap();
        let _p1 = service.get_all_states().await;
        assert_eq!(p.len(), 1);
    }
    #[tokio::test]
    async fn test_get_state_by_id() {
        let mut dao_mock = MockStateMasterDao::new();
        dao_mock.expect_get_all_states().times(1).returning(|| {
            vec![StateMasterModel {
                id: 1,
                state_name: StateName::new("Uttarakhand").unwrap(),
                audit_metadata: Default::default(),
                country_id:Uuid::now_v7()
            }]
        });
        let service = StateMasterServiceImpl {
            dao: Arc::new(dao_mock),
            cache_all: Cache::new(1),
            cache_by_id: Cache::new(40),
        };
        let p = service.get_state_by_id(1).await;
        let p1 = service.get_state_by_id(1).await;
        assert_that!(p).is_some();
        assert_that!(p1).is_some();
    }
}
