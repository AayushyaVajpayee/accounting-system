use std::fmt::Write;
use std::sync::Arc;
use actix_web::http::header::q;

use async_trait::async_trait;
use deadpool_postgres::Pool;
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::common_utils::pg_util::pg_util::ToPostgresString;
use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_json_at_index;
use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_uuid;
use crate::masters::product_item_master::product_item_db_models::{convert_db_resp_to_product_item_db_resp, ProductItemDb};
use crate::masters::product_item_master::product_item_models::ProductItemResponse;

#[async_trait]
pub trait ProductItemDao: Send + Sync {
    async fn create_product_item(&self, item: &ProductItemDb) -> Result<Uuid, DaoError>;
    async fn get_product(&self, product_id: Uuid, tenant_id: Uuid)
                         -> Result<Option<ProductItemResponse>, DaoError>;
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
        let mut simple_query = String::with_capacity(1500);
        write!(&mut simple_query, "begin transaction;\n")?;
        write!(&mut simple_query, "select create_product_item(")?;
        item.fmt_postgres(&mut simple_query)?;
        write!(&mut simple_query, ");\n commit;")?;
        let conn = self.postgres_client.get().await?;
        let rows = conn.simple_query(simple_query.as_str()).await?;
        let value = parse_db_output_of_insert_create_and_return_uuid(&rows)?;
        Ok(value)
    }

    async fn get_product(&self, product_id: Uuid, tenant_id: Uuid)
                         -> Result<Option<ProductItemResponse>, DaoError> {
        let query = format!(r#"
            select get_product_item('{}','{}');
        "#, product_id, tenant_id);
        let conn = self.postgres_client.get().await?;
        let rows = conn.simple_query(&q).await?;
        let value = parse_db_output_of_insert_create_and_return_json_at_index(&rows, 0)?;
        if let Some(value) = value {
            let product = convert_db_resp_to_product_item_db_resp(value)?;
            Ok(Some(product))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::accounting::postgres_factory::test_utils_postgres::get_dao_generic;
    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::masters::product_item_master::product_item_dao::{ProductItemDao, ProductItemDaoImpl};
    use crate::masters::product_item_master::product_item_db_models::convert_product_creation_request_to_product_item_db;
    use crate::masters::product_item_master::product_item_models::tests::{a_product_creation_request, SEED_PRODUCT_ITEM_ID};
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    async fn get_dao() -> ProductItemDaoImpl {
        get_dao_generic(|c| ProductItemDaoImpl { postgres_client: c }, None).await
    }

    #[tokio::test]
    async fn test_get_product_item() {
        let dao = get_dao().await;
        let _product_item = dao.get_product(*SEED_PRODUCT_ITEM_ID, *SEED_TENANT_ID).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_product_item_for_null_entry() {
        let dao = get_dao().await;
        let product_item = dao.get_product(Uuid::now_v7(), *SEED_TENANT_ID).await.unwrap();
        assert!(product_item.is_none())
    }

    #[tokio::test]
    async fn test_create_product_item() {
        let dao = get_dao().await;
        let req = a_product_creation_request(Default::default());
        let product_item_db = convert_product_creation_request_to_product_item_db(&req, *SEED_TENANT_ID, *SEED_USER_ID);
        let product_id = dao.create_product_item(&product_item_db).await.unwrap();
        let product_item = dao.get_product(product_id, *SEED_TENANT_ID).await.unwrap().unwrap();
        assert_eq!(product_item.temporal_cess_rates.len(), 1);
        assert_eq!(product_item.temporal_tax_rates.len(), 1);
        assert_eq!(product_item.base_master_fields.id, product_id);
    }
}