use std::sync::Arc;

use crate::common_utils::dao_error::DaoError;
use crate::common_utils::db_row_conversion_utils::convert_row_to_audit_metadata_base;
use crate::masters::state_master::state_models::{StateMasterModel, StateName};
use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::Pool;
#[cfg(test)]
use mockall::{automock, predicate::*};
use tokio_postgres::Row;
use uuid::Uuid;

const SELECT_FIELDS: &str =
    "id,state_name,state_code,country_id,created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "state_master";

const FETCH_ALL_QUERY: &str = concatcp!("select ", SELECT_FIELDS, " from ", TABLE_NAME);
const BY_ID_QUERY: &str = concatcp!(
    "select ",
    SELECT_FIELDS,
    " from ",
    TABLE_NAME,
    " where id =$1"
);
#[cfg_attr(test, automock)]
#[async_trait]
pub trait StateMasterDao: Send + Sync {
    async fn get_all_states(&self) -> Vec<StateMasterModel>;

    async fn get_state_by_id(&self, id: &Uuid) -> Option<StateMasterModel>;
}

struct StateMasterDaoPostgresImpl {
    postgres_client: Arc<Pool>,
}

impl TryFrom<&Row> for StateMasterModel {
    type Error = DaoError;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(StateMasterModel {
            id: row.get(0),
            state_name: StateName::new(row.get(1))?,
            state_code: row.get(2),
            country_id: row.get(3),
            audit_metadata: convert_row_to_audit_metadata_base(4, &row)?,
        })
    }
}
#[allow(dead_code)]
pub fn get_state_master_dao(client: Arc<Pool>) -> Arc<dyn StateMasterDao> {
    let state_master_dao = StateMasterDaoPostgresImpl {
        postgres_client: client,
    };
    Arc::new(state_master_dao)
}

#[async_trait]
impl StateMasterDao for StateMasterDaoPostgresImpl {
    async fn get_all_states(&self) -> Vec<StateMasterModel> {
        let conn = self.postgres_client.get().await.unwrap();
        let rows = conn.query(FETCH_ALL_QUERY, &[]).await.unwrap();
        rows.iter().map(|row| row.try_into().unwrap()).collect()
    }

    async fn get_state_by_id(&self, id: &Uuid) -> Option<StateMasterModel> {
        let conn = self.postgres_client.get().await.unwrap();
        let rows = conn.query(BY_ID_QUERY, &[&id]).await.unwrap();
        rows.iter().map(|row| row.try_into().unwrap()).next()
    }
}

#[cfg(test)]
mod tests {
    use speculoos::assert_that;
    use speculoos::prelude::OptionAssertions;

    use crate::accounting::postgres_factory::test_utils_postgres::{
        get_postgres_conn_pool, get_postgres_image_port,
    };
    use crate::masters::state_master::state_master_dao::{
        StateMasterDao, StateMasterDaoPostgresImpl,
    };
    use crate::masters::state_master::state_models::tests::SEED_STATE_ID;

    #[tokio::test]
    async fn should_be_able_to_fetch_all_states() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let state_master_dao = StateMasterDaoPostgresImpl {
            postgres_client: postgres_client.clone(),
        };
        let p = state_master_dao.get_all_states().await;
        assert!(!p.is_empty());
    }

    #[tokio::test]
    async fn should_be_able_fetch_state_by_id() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let state_master_dao = StateMasterDaoPostgresImpl {
            postgres_client: postgres_client.clone(),
        };
        let state = state_master_dao.get_state_by_id(&SEED_STATE_ID).await;
        assert_that!(state).is_some();
    }
}
