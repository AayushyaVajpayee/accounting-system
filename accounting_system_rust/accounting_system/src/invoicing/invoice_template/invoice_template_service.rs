use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use moka::future::Cache;
use thiserror::Error;
use uuid::Uuid;
use crate::common_utils::dao_error::DaoError;
use crate::invoicing::invoice_template::invoice_template_dao::{get_invoice_template_dao, InvoiceTemplateDao};
use crate::invoicing::invoice_template::invoice_template_models::InvoiceTemplateMaster;

#[derive(Debug, Error)]
pub enum InvoiceTemplateServiceError {
    #[error(transparent)]
    Db(#[from] DaoError)
}

type TemplateEntityOpt = Option<Arc<InvoiceTemplateMaster>>;

#[async_trait]
pub trait InvoiceTemplateService {
    async fn get_template_by_id(&self, id: Uuid, tenant_id: Uuid) -> Result<TemplateEntityOpt, InvoiceTemplateServiceError>;
    async fn is_valid_template_id(&self, id: Uuid, tenant_id: Uuid) -> Result<bool, InvoiceTemplateServiceError>;
}

struct InvoiceTemplateServiceImpl {
    dao: Arc<dyn InvoiceTemplateDao>,
    //(tenant_id,id)
    cache_by_tenant_id_and_id: Cache<(Uuid, Uuid), Arc<InvoiceTemplateMaster>>,
}

#[async_trait]
impl InvoiceTemplateService for InvoiceTemplateServiceImpl {
    async fn get_template_by_id(&self, id: Uuid, tenant_id: Uuid) -> Result<TemplateEntityOpt, InvoiceTemplateServiceError> {
        let key = (tenant_id, id);
        if let Some(entity) = self.cache_by_tenant_id_and_id.get(&key).await {
            return Ok(Some(entity));
        }
        let p = self.dao.get_invoice_template_by_id(&id, &tenant_id).await?;
        if let Some(entity) = p {
            let k = Arc::new(entity);
            self.cache_by_tenant_id_and_id.insert(key, k.clone()).await;
            Ok(Some(k))
        } else {
            Ok(None)
        }
    }

    async fn is_valid_template_id(&self, id: Uuid, tenant_id: Uuid) -> Result<bool, InvoiceTemplateServiceError> {
        let k = self.get_template_by_id(id, tenant_id).await?;
        Ok(k.is_some())
    }
}

pub fn get_invoice_template_master_service(arc: Arc<Pool>) -> Arc<dyn InvoiceTemplateService> {
    let dao = get_invoice_template_dao(arc);
    let cache: Cache<(Uuid, Uuid), Arc<InvoiceTemplateMaster>> =
        Cache::builder()
            .time_to_live(Duration::from_secs(300))
            .max_capacity(1000)
            .build();
    let service = InvoiceTemplateServiceImpl {
        dao,
        cache_by_tenant_id_and_id: cache,
    };
    Arc::new(service)
}



