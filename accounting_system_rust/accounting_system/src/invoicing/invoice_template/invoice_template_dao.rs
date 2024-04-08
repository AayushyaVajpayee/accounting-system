use std::sync::Arc;use std::fmt::Write;

use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::{GenericClient, Pool};
use tokio_postgres::Row;
use uuid::Uuid;
use crate::invoicing::invoice_template::invoice_template_models::CreateInvoiceTemplateRequest;

use crate::common_utils::dao_error::DaoError;
use crate::common_utils::db_row_conversion_utils::{
    convert_row_to_audit_metadata_base, convert_row_to_base_master_fields,
};
use crate::common_utils::pg_util::pg_util::ToPostgresString;
use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_uuid;
use crate::invoicing::invoice_template::invoice_template_models::{CreateInvoiceTemplateDbRequest, InvoiceTemplateMaster};

const TABLE_NAME: &str = "invoice_template";
const SELECT_FIELDS: &str = "id,entity_version_id,tenant_id,active,approval_status,remarks,sample_doc_s3_id,created_by,updated_by,created_at,updated_at";
const QUERY_BY_ID: &str = concatcp!(
    "select ",
    SELECT_FIELDS,
    " from ",
    TABLE_NAME,
    " where id=$1 and tenant_id=$2"
);

#[async_trait]
pub trait InvoiceTemplateDao: Send + Sync {
    async fn create_invoice_template(&self,request: CreateInvoiceTemplateRequest, tenant_id: Uuid, user_id: Uuid)
                                     -> Result<Uuid, DaoError>;
    async fn get_invoice_template_by_id(
        &self,
        id: &Uuid,
        tenant_id: &Uuid,
    ) -> Result<Option<InvoiceTemplateMaster>, DaoError>;
}

pub struct InvoiceTemplateDaoImpl {
    postgres_client: Arc<Pool>,
}

#[allow(dead_code)]
pub fn get_invoice_template_dao(client: Arc<Pool>) -> Arc<dyn InvoiceTemplateDao> {
    let a = InvoiceTemplateDaoImpl {
        postgres_client: client,
    };
    Arc::new(a)
}

impl TryFrom<Row> for InvoiceTemplateMaster {
    type Error = DaoError;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        let (base_master_fields, next_ind) = convert_row_to_base_master_fields(&row)?;
        Ok(InvoiceTemplateMaster {
            base_master_fields,
            sample_doc_s3_id: row.get(next_ind),
            audit_metadata: convert_row_to_audit_metadata_base(next_ind + 1, &row)?,
        })
    }
}

#[async_trait]
impl InvoiceTemplateDao for InvoiceTemplateDaoImpl {
    async fn create_invoice_template(&self,request: CreateInvoiceTemplateRequest,
                                     tenant_id: Uuid,
                                     user_id: Uuid) -> Result<Uuid, DaoError> {
        let re:CreateInvoiceTemplateDbRequest=CreateInvoiceTemplateDbRequest{
            idempotence_key: request.idempotence_key,
            sample_doc_s3_id: request.sample_doc_s3_id,
            tenant_id,
            user_id,
        };
        let mut simple_query = String::with_capacity(1500);
        write!(&mut simple_query,"begin transaction;select create_invoice_template(")?;
        re.fmt_postgres(&mut simple_query)?;
        write!(&mut simple_query,");commit;")?;
        let conn = self.postgres_client.get().await?;
        let rows = conn.simple_query(simple_query.as_str()).await?;
        parse_db_output_of_insert_create_and_return_uuid(&rows)
    }

    async fn get_invoice_template_by_id(
        &self,
        id: &Uuid,
        tenant_id: &Uuid,
    ) -> Result<Option<InvoiceTemplateMaster>, DaoError> {
        let query = QUERY_BY_ID;
        let entity = self
            .postgres_client
            .get()
            .await?
            .query_opt(query, &[&id, &tenant_id])
            .await?
            .map(|a| a.try_into())
            .transpose()?;
        Ok(entity)
    }
}

#[cfg(test)]
mod tests {
    use speculoos::assert_that;
    use speculoos::option::OptionAssertions;
    use uuid::Uuid;

    use crate::accounting::postgres_factory::test_utils_postgres::{
        get_dao_generic, get_postgres_conn_pool, get_postgres_image_port,
    };
    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::invoicing::invoice_template::invoice_template_dao::{
        InvoiceTemplateDao, InvoiceTemplateDaoImpl,
    };
    use crate::invoicing::invoice_template::invoice_template_models::CreateInvoiceTemplateRequest;
    use crate::invoicing::invoice_template::invoice_template_models::tests::SEED_INVOICE_TEMPLATE_ID;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;
    #[tokio::test]
    async fn test_create_invoice_template(){
        let dao = get_dao_generic(
            |a| InvoiceTemplateDaoImpl {
                postgres_client: a.clone(),
            },
            None,
        )
            .await;
        let id = dao.create_invoice_template(CreateInvoiceTemplateRequest{
            idempotence_key: Uuid::now_v7(),
            sample_doc_s3_id: None,
        },*SEED_TENANT_ID,*SEED_USER_ID).await.unwrap();
        let template = dao.get_invoice_template_by_id(&id,&*SEED_TENANT_ID).await.unwrap();
        assert_that!(template).is_some();
        
    }
    #[tokio::test]
    async fn test_get_invoice_template() {
        let dao = get_dao_generic(
            |a| InvoiceTemplateDaoImpl {
                postgres_client: a.clone(),
            },
            None,
        )
            .await;
        let template_id = *SEED_INVOICE_TEMPLATE_ID;
        let tenant_id = *SEED_TENANT_ID;
        let a = dao
            .get_invoice_template_by_id(&template_id, &tenant_id)
            .await
            .unwrap();
        assert_that!(a).is_some();
    }
}
