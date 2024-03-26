use std::sync::Arc;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use uuid::Uuid;
use crate::common_utils::dao_error::DaoError;
use crate::masters::product_item_master::product_item_db_models::ProductItemDb;

#[async_trait]
pub trait ProductItemDao: Send + Sync {
    async fn create_product_item(&self, item: &ProductItemDb) -> Result<Uuid, DaoError>;
}


struct ProductItemDaoImpl {
    postgres_client: Arc<Pool>,
}

pub fn get_product_item_dao(client: Arc<Pool>)
                            -> Box<dyn ProductItemDao> {
    Box::new(ProductItemDaoImpl {
        postgres_client: client
    })
}

#[async_trait]
impl ProductItemDao for ProductItemDaoImpl {
    async fn create_product_item(&self, item: &ProductItemDb) -> Result<Uuid, DaoError> {
        
        todo!()
    }
}