use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use std::sync::Arc;
use std::time::Duration;
use deadpool_postgres::Pool;
use moka::future::Cache;
use thiserror::Error;
use uuid::Uuid;
use crate::common_utils::cache_utils::get_or_fetch_entity;

use crate::common_utils::dao_error::DaoError;
use crate::invoicing::invoicing_series::invoicing_series_dao::{get_invoicing_series_dao, InvoicingSeriesDao};
use crate::invoicing::invoicing_series::invoicing_series_models::{CreateInvoiceNumberSeriesRequest, InvoicingSeriesMaster};

#[derive(Debug, Error)]
pub enum InvoicingSeriesServiceError {
    #[error(transparent)]
    Db(#[from] DaoError)
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait InvoicingSeriesService: Send + Sync {
    async fn create_invoice_series(&self, request: &CreateInvoiceNumberSeriesRequest) -> Result<Uuid, InvoicingSeriesServiceError>;

    async fn get_invoicing_series_by_id(&self, invoicing_series_id: Uuid,tenant_id:Uuid) -> Result<Option<Arc<InvoicingSeriesMaster>>, InvoicingSeriesServiceError>;
    async fn is_valid_invoicing_series_id(&self,invoicing_series_id:Uuid,tenant_id:Uuid)->Result<bool,InvoicingSeriesServiceError>;
}

struct InvoicingSeriesServiceImpl {
    dao: Arc<dyn InvoicingSeriesDao>,
    cache:Cache<(Uuid,Uuid),Arc<InvoicingSeriesMaster>>
}
#[allow(dead_code)]
pub  fn get_invoicing_series_service(pool:Arc<Pool>)->Arc<dyn InvoicingSeriesService>{
    let invoicing_series_dao=get_invoicing_series_dao(pool);
    let cache:Cache<(Uuid,Uuid),Arc<InvoicingSeriesMaster>> = Cache::builder()
        .max_capacity(1000)
        .time_to_live(Duration::from_secs(300))
        .build();
    let invoicing_series_service = InvoicingSeriesServiceImpl{
        dao:invoicing_series_dao,
        cache
    };
    Arc::new(invoicing_series_service)
}

#[async_trait]
impl InvoicingSeriesService for InvoicingSeriesServiceImpl {
    async fn create_invoice_series(&self, request: &CreateInvoiceNumberSeriesRequest) -> Result<Uuid, InvoicingSeriesServiceError> {
        let d = self.dao.create_invoice_series(&request).await?;
        Ok(d)
    }

    async fn get_invoicing_series_by_id(&self, invoicing_series_id: Uuid,tenant_id:Uuid) -> Result<Option<Arc<InvoicingSeriesMaster>>, InvoicingSeriesServiceError> {
        let fetch = async{
            let p =self.dao.get_invoicing_series_by_id(invoicing_series_id,tenant_id).await?;
            Ok(p)
        };
        get_or_fetch_entity(tenant_id,invoicing_series_id,&self.cache,fetch).await
    }

    async fn is_valid_invoicing_series_id(&self, invoicing_series_id: Uuid, tenant_id: Uuid) -> Result<bool, InvoicingSeriesServiceError> {
        let p = self
            .get_invoicing_series_by_id(invoicing_series_id,tenant_id)
            .await?;
        Ok(p.is_some())
    }
}



