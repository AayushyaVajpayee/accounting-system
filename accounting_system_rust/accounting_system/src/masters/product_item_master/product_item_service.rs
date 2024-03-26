use async_trait::async_trait;
use uuid::Uuid;
use crate::common_utils::dao_error::DaoError;
use crate::common_utils::utils::TenantId;
use crate::masters::product_item_master::product_item_dao::ProductItemDao;
use crate::masters::product_item_master::product_item_db_models::convert_product_creation_request_to_product_item_db;
use crate::masters::product_item_master::product_item_models::ProductCreationRequest;

#[async_trait]
pub trait ProductItemService {
    async fn create_product(&self, request: ProductCreationRequest, tenant_id: Uuid, created_by: Uuid) -> anyhow::Result<()>;
}


struct ProductItemServiceImpl {
    dao: Box<dyn ProductItemDao>,
}

struct ProductItemDbResponse {}

#[async_trait]
impl ProductItemService for ProductItemServiceImpl {
    async fn create_product(&self, request: ProductCreationRequest, tenant_id: Uuid, created_by: Uuid)
                            -> anyhow::Result<()> {
        let p = convert_product_creation_request_to_product_item_db(&request,
                                                                    tenant_id,
                                                                    created_by);
        let pdf = self.dao.create_product_item(&p).await.unwrap();
        Ok(())
    }
}