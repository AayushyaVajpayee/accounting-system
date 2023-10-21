use std::sync::Arc;
use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::Pool;
#[cfg(test)]
use mockall::automock;
use tokio_postgres::Row;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::pincode_master::pincode_models::{Pincode, PincodeMaster};

const SELECT_FIELDS: &str = "id,pincode,city_id,created_by,updated_by,created_at,updated_at,country_id";

const TABLE_NAME: &str = "pincode_master";

const FETCH_ALL_QUERY: &str = concatcp!("select ", SELECT_FIELDS, " from ", TABLE_NAME);

const BY_ID_QUERY: &str = concatcp!("select ", SELECT_FIELDS, " from ", TABLE_NAME," where id=$1");

#[cfg_attr(test, automock)]
#[async_trait]
pub trait PincodeMasterDao:Send+Sync {
    async fn get_all_pincodes(&self) -> Vec<PincodeMaster>;

    async fn get_pincode_by_id(&self, id: i32) -> Option<PincodeMaster>;
}

struct PincodeMasterDaoImpl {
    postgres_client: &'static Pool,
}

impl TryFrom<&Row> for PincodeMaster {
    type Error = &'static str;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(PincodeMaster {
            id: row.get(0),
            pincode: Pincode::new(row.get(1),row.get(7))?,
            city_id: row.get(2),
            audit_metadata: AuditMetadataBase {
                created_by: row.get(3),
                updated_by: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
            },
            country_id:row.get(7)
        })
    }
}

#[async_trait]
impl PincodeMasterDao for PincodeMasterDaoImpl {
    async fn get_all_pincodes(&self) -> Vec<PincodeMaster> {
        let conn = self.postgres_client.get().await.unwrap();
        let rows = conn.query(FETCH_ALL_QUERY, &[]).await.unwrap();
        rows.iter().map(|row| row.try_into().unwrap()).collect()
    }

    async fn get_pincode_by_id(&self, id: i32) -> Option<PincodeMaster> {
        let conn = self.postgres_client.get().await.unwrap();
        let rows = conn.query(BY_ID_QUERY, &[&id]).await.unwrap();
        rows.iter().map(|row| row.try_into().unwrap()).next()
    }
}

pub fn get_pincode_master_dao(client: &'static Pool)->Arc<dyn PincodeMasterDao>{
    let pincode_master_dao = PincodeMasterDaoImpl{
        postgres_client:client
    };
    Arc::new(pincode_master_dao)
}

#[cfg(test)]
mod tests {
    use spectral::assert_that;
    use spectral::option::OptionAssertions;

    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::masters::pincode_master::pincode_master_dao::{PincodeMasterDao, PincodeMasterDaoImpl};

    #[tokio::test]
    async fn should_be_able_to_fetch_all_pincodes() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let pincode_master_dao = PincodeMasterDaoImpl{postgres_client};
        let pincodes = pincode_master_dao.get_all_pincodes().await;
       assert!(!pincodes.is_empty())
    }

    #[tokio::test]
    async fn should_be_able_to_fetch_pincode_by_id(){
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let pincode_master_dao = PincodeMasterDaoImpl{postgres_client};
        let pincode = pincode_master_dao.get_pincode_by_id(1).await;
        assert_that!(pincode).is_some();
    }
}
