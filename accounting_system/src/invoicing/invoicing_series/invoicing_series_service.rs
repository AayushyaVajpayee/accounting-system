use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use thiserror::Error;
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::invoicing::invoicing_series::invoicing_series_dao::InvoicingSeriesDao;
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

    async fn get_invoicing_series_by_id(&self, invoicing_series_id: &Uuid) -> Result<Option<InvoicingSeriesMaster>, InvoicingSeriesServiceError>;
}

struct InvoicingSeriesServiceImpl {
    dao: Arc<dyn InvoicingSeriesDao>,
}


#[async_trait]
impl InvoicingSeriesService for InvoicingSeriesServiceImpl {
    async fn create_invoice_series(&self, request: &CreateInvoiceNumberSeriesRequest) -> Result<Uuid, InvoicingSeriesServiceError> {
        let d = self.dao.create_invoice_series(&request).await?;
        Ok(d)
    }

    async fn get_invoicing_series_by_id(&self, invoicing_series_id: &Uuid) -> Result<Option<InvoicingSeriesMaster>, InvoicingSeriesServiceError> {
        let opt_in_ser = self.dao.get_invoicing_series_by_id(invoicing_series_id).await?;
        Ok(opt_in_ser)
    }
}



