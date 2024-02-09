use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::Pool;
use std::sync::Arc;
use tokio_postgres::Row;
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::common_utils::db_row_conversion_utils::{convert_row_to_audit_metadata_base, convert_row_to_base_master_fields};
use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_uuid;
use crate::invoicing::invoicing_series::invoicing_series_models::{CreateInvoiceNumberSeriesRequest, InvoiceNumberPrefix, InvoicingSeriesMaster, InvoicingSeriesName};

const TABLE_NAME: &str = "invoicing_series_mst";
const SELECT_FIELDS: &str = "id,entity_version_id,tenant_id,active,approval_status,remarks,name,prefix,zero_padded_counter,created_by,updated_by,created_at,updated_at";

const QUERY_BY_ID: &str = concatcp!("select ",SELECT_FIELDS," from ",TABLE_NAME, " where id = $1 and tenant_id = $2");

#[async_trait]
pub trait InvoicingSeriesDao: Send + Sync {
    async fn create_invoice_series(&self, request: &CreateInvoiceNumberSeriesRequest) -> Result<Uuid, DaoError>;
    async fn get_invoicing_series_by_id(&self, invoicing_series_id: Uuid,tenant_id:Uuid) -> Result<Option<InvoicingSeriesMaster>, DaoError>;
}

impl TryFrom<Row> for InvoicingSeriesMaster {
    type Error = DaoError;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        let (base_master_fields, next_ind) = convert_row_to_base_master_fields(&row)?;
        let h = InvoicingSeriesMaster {
            base_master_fields,
            name: InvoicingSeriesName::new(row.try_get(next_ind)?)?,
            prefix: InvoiceNumberPrefix::new(row.try_get(next_ind + 1)?)?,
            zero_padded_counter: row.try_get(next_ind + 2)?,
            audit_metadata: convert_row_to_audit_metadata_base(next_ind + 3, &row)?,
        };
        Ok(h)
    }
}


struct InvoicingSeriesDaoImpl {
    postgres_client: Arc<Pool>,
}

pub fn get_invoicing_series_dao(client: Arc<Pool>) -> Arc<dyn InvoicingSeriesDao> {
    let ad = InvoicingSeriesDaoImpl {
        postgres_client: client
    };
    Arc::new(ad)
}

#[async_trait]
impl InvoicingSeriesDao for InvoicingSeriesDaoImpl {
    async fn create_invoice_series(&self, request: &CreateInvoiceNumberSeriesRequest) -> Result<Uuid, DaoError> {
        let simple_query = format!(
            r#"
           begin transaction;
           select create_invoice_series(Row('{}','{}','{}','{}',{},{},{},'{}'));
           commit;
           "#, request.idempotence_key,
            request.tenant_id,
            request.name.inner(),
            request.prefix.inner(),
            request.zero_padded_counter,
            request.start_value.unwrap_or(0),
            request.financial_year.inner(),
            request.created_by
        );
        let conn = self.postgres_client.get().await?;
        let rows = conn.simple_query(simple_query.as_str()).await?;
        parse_db_output_of_insert_create_and_return_uuid(&rows)
    }

    async fn get_invoicing_series_by_id(&self, invoicing_series_id: Uuid,tenant_id:Uuid) -> Result<Option<InvoicingSeriesMaster>, DaoError> {
        let query = QUERY_BY_ID;
        let jj = self.postgres_client.get().await?
            .query_opt(query, &[&invoicing_series_id,&tenant_id]).await?
            .map(|a| a.try_into())
            .transpose()?;
        Ok(jj)
    }
}


#[cfg(test)]
mod tests {
    use spectral::assert_that;
    use spectral::option::OptionAssertions;

    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::invoicing::invoicing_series::invoicing_series_dao::{InvoicingSeriesDao, InvoicingSeriesDaoImpl};
    use crate::invoicing::invoicing_series::invoicing_series_models::tests::a_create_invoice_number_series_request;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    #[tokio::test]
    async fn test_insert_invoicing_series() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let dao = InvoicingSeriesDaoImpl { postgres_client: postgres_client.clone() };
        let in_series = a_create_invoice_number_series_request(Default::default());
        let p = dao.create_invoice_series(&in_series).await.unwrap();
        let jj = dao.get_invoicing_series_by_id(p,*SEED_TENANT_ID).await.unwrap();
        assert_that!(jj).is_some();
    }
}