use std::sync::Arc;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use uuid::Uuid;
use crate::common_utils::dao_error::DaoError;
use crate::common_utils::pg_util::pg_util::ToPostgresString;
use crate::masters::product_item_master::product_item_db_models::{ProductItemDb, ProductItemDbResponse};
use std::fmt::Write;
use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_uuid;

#[async_trait]
pub trait ProductItemDao: Send + Sync {
    async fn create_product_item(&self, item: &ProductItemDb) -> Result<Uuid, DaoError>;
    async fn get_product(&self, product_id: Uuid, tenant_id: Uuid)
                         -> Result<ProductItemDbResponse, DaoError> ;
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
                         -> Result<ProductItemDbResponse, DaoError>  {
        let j =r#"
        
        "#;
        todo!()
    }
}