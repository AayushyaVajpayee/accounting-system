use std::sync::Arc;

use async_trait::async_trait;
use moka::future::Cache;

use crate::accounting::postgres_factory::get_postgres_conn_pool;
use crate::masters::state_master::state_master_dao::{get_state_master_dao, StateMasterDao};
use crate::masters::state_master::state_models::StateMasterModel;

const CACHE_ALL_KEY: i32 = 1;
#[async_trait]
pub trait StateMasterService {
    async fn get_all_states(&self) -> Arc<Vec<Arc<StateMasterModel>>>;
    async fn get_state_by_id(&self, id: i32) -> Option<Arc<StateMasterModel>>;
}

struct StateMasterServiceImpl {
    dao: Box<dyn StateMasterDao + Send + Sync>,
    cache_by_id: Cache<i32, Arc<StateMasterModel>>,
    cache_all: Cache<i32, Arc<Vec<Arc<StateMasterModel>>>>,
}
#[allow(dead_code)]
pub async fn get_state_master_service() -> Box<dyn StateMasterService + Send + Sync> {
    let pclient = get_postgres_conn_pool();
    let state_master_dao = get_state_master_dao(pclient);
    let cache: Cache<i32, Arc<Vec<Arc<StateMasterModel>>>> = Cache::new(40);
    cache.insert(CACHE_ALL_KEY, Arc::new(vec![])).await;
    let state_master_s = StateMasterServiceImpl {
        dao: state_master_dao,
        cache_by_id: Cache::new(40),
        cache_all: cache,
    };
    Box::new(state_master_s)
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
    async fn get_all_states(&self) -> Arc<Vec<Arc<StateMasterModel>>> {
        let cache = self.cache_all.clone();
        if cache.get(&CACHE_ALL_KEY).await.is_none() {
            self.populate_caches().await;
        }
        cache.get(&CACHE_ALL_KEY).await.unwrap() //safe to call unwrap because we are initialising with empty vector
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
