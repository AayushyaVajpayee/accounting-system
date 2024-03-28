use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::masters::product_item_master::product_item_dao::ProductItemDao;
use crate::masters::product_item_master::product_item_db_models::convert_product_creation_request_to_product_item_db;
use crate::masters::product_item_master::product_item_models::{ProductCreationRequest, ProductItemResponse};

#[derive(Debug, Error)]
pub enum ProductItemServiceError {
    #[error("error in db {0}")]
    Db(#[from]DaoError),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

#[async_trait]
pub trait ProductItemService {
    async fn create_product(&self, request: ProductCreationRequest, tenant_id: Uuid,
                            created_by: Uuid)
                            -> Result<Uuid, ProductItemServiceError>;

    async fn get_product(&self, product_id: Uuid, tenant_id: Uuid)
                         -> Result<ProductItemResponse, ProductItemServiceError>;
}


struct ProductItemServiceImpl {
    dao: Box<dyn ProductItemDao>,
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
                         -> Result<ProductItemResponse, ProductItemServiceError> {
        let product_item_response = self.dao.get_product(product_id, tenant_id)
            .await?;
        Ok(product_item_response)
    }
}