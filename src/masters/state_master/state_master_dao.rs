use std::sync::Arc;
use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::Pool;
#[cfg(test)]
use mockall::{automock, predicate::*};
use tokio_postgres::Row;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::state_master::state_models::{StateMasterModel, StateName};

const SELECT_FIELDS: &str = "id,state_name,created_by,updated_by,created_at,updated_at,country_id";
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
pub trait StateMasterDao:Send+Sync {
    async fn get_all_states(&self) -> Vec<StateMasterModel>;

    async fn get_state_by_id(&self, id: i32) -> Option<StateMasterModel>;
}

struct StateMasterDaoPostgresImpl {
    postgres_client: &'static Pool
}

impl TryFrom<&Row> for StateMasterModel {
    type Error = &'static str;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(StateMasterModel {
            id: row.get(0),
            state_name: StateName::new(row.get(1))?,
            audit_metadata: AuditMetadataBase {
                created_by: row.get(2),
                updated_by: row.get(3),
                created_at: row.get(4),
                updated_at: row.get(5),
            },
            country_id:row.get(6)
        })
    }
}

pub fn get_state_master_dao(client: &'static Pool) -> Arc<dyn StateMasterDao> {
    let state_master_dao = StateMasterDaoPostgresImpl{
        postgres_client:client
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

    async fn get_state_by_id(&self, id: i32) -> Option<StateMasterModel> {
        let conn = self.postgres_client.get().await.unwrap();
        let rows = conn.query(BY_ID_QUERY, &[&id]).await.unwrap();
        rows.iter().map(|row| row.try_into().unwrap()).next()
    }
}

#[cfg(test)]
mod tests {
    use spectral::assert_that;
    use spectral::prelude::OptionAssertions;

    use crate::accounting::postgres_factory::test_utils_postgres::{
        get_postgres_conn_pool, get_postgres_image_port,
        };
    use crate::masters::state_master::state_master_dao::{
        StateMasterDao, StateMasterDaoPostgresImpl,
    };

    #[tokio::test]
    async fn should_be_able_to_fetch_all_states() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let state_master_dao = StateMasterDaoPostgresImpl { postgres_client };
        let p = state_master_dao.get_all_states().await;
        assert!(!p.is_empty());
    }

    #[tokio::test]
    async fn should_be_able_fetch_state_by_id() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let state_master_dao = StateMasterDaoPostgresImpl { postgres_client };
        let state = state_master_dao.get_state_by_id(0).await;
        assert_that!(state).is_some();
    }
}
