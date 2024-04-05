use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use deadpool_postgres::Pool;
#[cfg(test)]
use mockall::automock;
use moka::future::Cache;
use thiserror::Error;
use uuid::Uuid;

use crate::common_utils::cache_utils::get_or_fetch_entity;
use crate::common_utils::dao_error::DaoError;
use crate::masters::product_item_master::product_item_dao::{get_product_item_dao, ProductItemDao};
use crate::masters::product_item_master::product_item_db_models::convert_product_creation_request_to_product_item_db;
use crate::masters::product_item_master::product_item_models::{
    ProductCreationRequest, ProductItemResponse,
};

#[derive(Debug, Error)]
pub enum ProductItemServiceError {
    #[error("error in db {0}")]
    Db(#[from] DaoError),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ProductItemService: Send + Sync {
    async fn create_product(
        &self,
        request: ProductCreationRequest,
        tenant_id: Uuid,
        created_by: Uuid,
    ) -> Result<Uuid, ProductItemServiceError>;

    async fn get_product(
        &self,
        product_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Option<Arc<ProductItemResponse>>, ProductItemServiceError>;

    async fn get_products(
        &self,
        product_ids: Vec<Uuid>,
        tenant_id: Uuid,
    ) -> Result<Vec<Arc<ProductItemResponse>>, ProductItemServiceError>;
}

struct ProductItemServiceImpl {
    dao: Box<dyn ProductItemDao>,
    //tenant_id and product_item_id
    cache_by_tenant_id_and_product_id: Cache<(Uuid, Uuid), Arc<ProductItemResponse>>,
}

pub fn get_product_item_service(pool: Arc<Pool>) -> Arc<dyn ProductItemService> {
    let dao = get_product_item_dao(pool);
    let cache: Cache<(Uuid, Uuid), Arc<ProductItemResponse>> = Cache::builder()
        .time_to_live(Duration::from_secs(300))
        .max_capacity(1000)
        .build();
    let service = ProductItemServiceImpl {
        dao,
        cache_by_tenant_id_and_product_id: cache,
    };
    Arc::new(service)
}

#[async_trait]
impl ProductItemService for ProductItemServiceImpl {
    async fn create_product(
        &self,
        request: ProductCreationRequest,
        tenant_id: Uuid,
        created_by: Uuid,
    ) -> Result<Uuid, ProductItemServiceError> {
        let p =
            convert_product_creation_request_to_product_item_db(&request, tenant_id, created_by);
        let product_id = self.dao.create_product_item(&p).await?;
        Ok(product_id)
    }

    async fn get_product(
        &self,
        product_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Option<Arc<ProductItemResponse>>, ProductItemServiceError> {
        let fetch = async {
            let p = self.dao.get_product(product_id, tenant_id).await?;
            Ok(p)
        };
        get_or_fetch_entity(
            tenant_id,
            product_id,
            &self.cache_by_tenant_id_and_product_id,
            fetch,
        )
        .await
    }
    async fn get_products(
        &self,
        product_ids: Vec<Uuid>,
        tenant_id: Uuid,
    ) -> Result<Vec<Arc<ProductItemResponse>>, ProductItemServiceError> {
        let mut list: Vec<(Uuid, Option<Arc<ProductItemResponse>>)> =
            Vec::with_capacity(product_ids.len());
        for product_id in product_ids {
            let a = self
                .cache_by_tenant_id_and_product_id
                .get(&(tenant_id, product_id))
                .await;
            list.push((product_id, a));
        }
        let (cached_products, uncached_product_ids): (Vec<_>, Vec<_>) =
            list.into_iter().partition(|a| a.1.is_some());
        let cached_products: Vec<Arc<ProductItemResponse>> =
            cached_products.into_iter().map(|a| a.1.unwrap()).collect();
        let uncached_product_ids: Vec<Uuid> =
            uncached_product_ids.into_iter().map(|a| a.0).collect();
        let db_products = self
            .dao
            .get_products(&uncached_product_ids, tenant_id)
            .await?;
        let mut combined_products: Vec<Arc<ProductItemResponse>> = cached_products;
        for product in db_products {
            let product_id = product.base_master_fields.id.clone();
            let product_arc = Arc::new(product);
            self.cache_by_tenant_id_and_product_id
                .insert((tenant_id, product_id), product_arc.clone())
                .await;
            combined_products.push(product_arc);
        }
        Ok(combined_products)
    }
}
