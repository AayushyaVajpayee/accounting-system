use std::sync::Arc;
use async_trait::async_trait;
use deadpool_postgres::Pool;
#[cfg(test)]
use mockall::automock;
use thiserror::Error;
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::masters::product_item_master::product_item_dao::{get_product_item_dao, ProductItemDao};
use crate::masters::product_item_master::product_item_db_models::convert_product_creation_request_to_product_item_db;
use crate::masters::product_item_master::product_item_models::{ProductCreationRequest, ProductItemResponse};

#[derive(Debug, Error)]
pub enum ProductItemServiceError {
    #[error("error in db {0}")]
    Db(#[from]DaoError),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ProductItemService: Send + Sync {
    async fn create_product(&self, request: ProductCreationRequest, tenant_id: Uuid,
                            created_by: Uuid)
                            -> Result<Uuid, ProductItemServiceError>;

    async fn get_product(&self, product_id: Uuid, tenant_id: Uuid)
                         -> Result<Option<ProductItemResponse>, ProductItemServiceError>;
}


struct ProductItemServiceImpl {
    dao: Box<dyn ProductItemDao>,
}

pub fn get_product_item_service(pool: Arc<Pool>) -> Arc<dyn ProductItemService> {
    let dao = get_product_item_dao(pool);
    let service = ProductItemServiceImpl {
        dao
    };
    Arc::new(service)
}


#[async_trait]
impl ProductItemService for ProductItemServiceImpl {
    async fn create_product(&self, request: ProductCreationRequest, tenant_id: Uuid, created_by: Uuid)
                            -> Result<Uuid, ProductItemServiceError> {
        let p = convert_product_creation_request_to_product_item_db(&request,
                                                                    tenant_id,
                                                                    created_by);
        let product_id = self.dao.create_product_item(&p).await?;
        Ok(product_id)
    }

    async fn get_product(&self, product_id: Uuid, tenant_id: Uuid)
                         -> Result<Option<ProductItemResponse>, ProductItemServiceError> {
        let product_item_response = self.dao.get_product(product_id, tenant_id)
            .await?;
        Ok(product_item_response)
    }
}