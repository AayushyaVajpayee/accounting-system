use std::sync::Arc;
use async_trait::async_trait;
use moka::future::Cache;
use thiserror::Error;
use uuid::Uuid;
use crate::common_utils::dao_error::DaoError;
use crate::invoicing::invoice_template::invoice_template_dao::InvoiceTemplateDao;
use crate::invoicing::invoice_template::invoice_template_models::InvoiceTemplateMaster;

#[derive(Debug, Error)]
pub enum InvoiceTemplateServiceError {
    #[error(transparent)]
    Db(#[from] DaoError)
}

type TemplateEntityOpt = Option<Arc<InvoiceTemplateMaster>>;

#[async_trait]
pub trait InvoiceTemplateService {
    async fn get_template_by_id(&self, id: &Uuid, tenant_id: &Uuid) -> Result<TemplateEntityOpt, InvoiceTemplateServiceError>;
}

struct InvoiceTemplateServiceImpl {
    dao: Arc<dyn InvoiceTemplateDao>,
    //(tenant_id,id)
    cache_by_tenant_id_and_id: Cache<(Uuid, Uuid), Arc<InvoiceTemplateMaster>>,
}
#[async_trait]
impl InvoiceTemplateService for InvoiceTemplateServiceImpl {
    async fn get_template_by_id(&self, id: &Uuid, tenant_id: &Uuid) -> Result<TemplateEntityOpt, InvoiceTemplateServiceError> {
        let key = (*tenant_id, *id);
        if let Some(entity) = self.cache_by_tenant_id_and_id.get(&key).await {
            return Ok(Some(entity));
        }
        let p = self.dao.get_invoice_template_by_id(id, tenant_id).await?;
        if let Some(entity) = p {
            let k = Arc::new(entity);
            self.cache_by_tenant_id_and_id.insert(key, k.clone()).await;
            Ok(Some(k))
        } else {
            Ok(None)
        }
    }
}



