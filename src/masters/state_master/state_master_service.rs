use crate::accounting::postgres_factory::get_postgres_conn_pool;
use crate::masters::state_master::state_master_dao::{get_state_master_dao, StateMasterDao};
use async_trait::async_trait;
use crate::masters::state_master::state_models::StateMasterModel;

#[async_trait]
pub trait StateMasterService {
    async fn get_all_states(&self)->Vec<StateMasterModel>;

    async fn get_state_by_id(&self,id:i32)->Option<StateMasterModel>;


}

struct StateMasterServiceImpl {
    state_master_dao: Box<dyn StateMasterDao + Send + Sync>,
}

pub fn get_state_master_service() -> Box<dyn StateMasterService + Send + Sync> {
    let pclient = get_postgres_conn_pool();
    let state_master_dao = get_state_master_dao(pclient);
    let state_master_s = StateMasterServiceImpl { state_master_dao };
    Box::new(state_master_s)
}

#[async_trait]
impl StateMasterService for StateMasterServiceImpl{
    async fn get_all_states(&self)->Vec<StateMasterModel> {
        self.state_master_dao.get_all_states().await
    }

    async fn get_state_by_id(&self,id:i32)->Option<StateMasterModel> {
        self.state_master_dao.get_state_by_id(id).await
    }


}