use std::sync::Arc;
use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::{GenericClient, Pool};
use tokio_postgres::Row;
use uuid::Uuid;
use crate::common_utils::dao_error::DaoError;
use crate::common_utils::db_row_conversion_utils::{convert_row_to_audit_metadata_base, convert_row_to_base_master_fields};
use crate::invoicing::invoice_template::invoice_template_models::InvoiceTemplateMaster;

const TABLE_NAME: &str = "invoice_template";
const SELECT_FIELDS: &str = "id,entity_version_id,tenant_id,active,approval_status,remarks,sample_doc_s3_id,created_by,updated_by,created_at,updated_at";
const QUERY_BY_ID: &str = concatcp!("select ",SELECT_FIELDS," from ",TABLE_NAME," where id=$1 and tenant_id=$2");

#[async_trait]
pub trait InvoiceTemplateDao: Send+Sync{
    async fn get_invoice_template_by_id(&self,id:&Uuid,tenant_id:&Uuid)->Result<Option<InvoiceTemplateMaster>,DaoError>;
}
pub struct InvoiceTemplateDaoImpl{
    postgres_client: Arc<Pool>,
}

#[allow(dead_code)]
pub fn get_invoice_template_dao(client:Arc<Pool>) ->Arc<dyn InvoiceTemplateDao>{
    let a = InvoiceTemplateDaoImpl{
        postgres_client:client
    };
    Arc::new(a)
}
impl TryFrom<Row> for InvoiceTemplateMaster{
    type Error = DaoError;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        let (base_master_fields,next_ind) = convert_row_to_base_master_fields(&row)?;
        Ok(InvoiceTemplateMaster{
            base_master_fields,
            sample_doc_s3_id: row.get(next_ind),
            audit_metadata: convert_row_to_audit_metadata_base(next_ind+1,&row)?,
        })
    }
}


#[async_trait]
impl InvoiceTemplateDao for InvoiceTemplateDaoImpl{
    async fn get_invoice_template_by_id(&self, id: &Uuid,tenant_id:&Uuid) -> Result<Option<InvoiceTemplateMaster>, DaoError> {
        let query = QUERY_BY_ID;
        let entity = self.postgres_client.get().await?
            .query_opt(query,&[&id,&tenant_id]).await?
            .map(|a|a.try_into())
            .transpose()?;
        Ok(entity)
    }
}
#[cfg(test)]
mod tests{
    use spectral::assert_that;
    use spectral::option::OptionAssertions;
    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::invoicing::invoice_template::invoice_template_dao::{InvoiceTemplateDao, InvoiceTemplateDaoImpl};
    use crate::invoicing::invoice_template::invoice_template_models::tests::SEED_INVOICE_TEMPLATE_ID;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    #[tokio::test]
    async fn test_get_invoice_template(){
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let dao= InvoiceTemplateDaoImpl{postgres_client};
        let template_id=*SEED_INVOICE_TEMPLATE_ID;
        let tenant_id = *SEED_TENANT_ID;
        let a = dao
            .get_invoice_template_by_id(&template_id,&tenant_id)
            .await.unwrap();
        assert_that!(a).is_some();
    }
}