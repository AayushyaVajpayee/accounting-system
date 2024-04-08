use std::sync::Arc;
use actix_web::{HttpResponseBuilder, Responder, ResponseError, web};
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use uuid::Uuid;
use crate::setup_routes;
use crate::invoicing::invoice_template::invoice_template_models::CreateInvoiceTemplateRequest;
use crate::common_utils::utils::{TenantId, UserId};
use crate::invoicing::invoice_template::invoice_template_service::{InvoiceTemplateService, InvoiceTemplateServiceError};

impl ResponseError for InvoiceTemplateServiceError {}


async fn create_invoice_template(
    data: Data<Arc<dyn InvoiceTemplateService>>,
    request: web::Json<CreateInvoiceTemplateRequest>,
    tenant_id: TenantId,
    user_id: UserId,
) -> actix_web::Result<impl Responder> {
    let template_id = data
        .create_template(request.into_inner(), tenant_id.inner(), user_id.inner())
        .await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(template_id))
}


async fn get_invoice_template_by_id(
    data: Data<Arc<dyn InvoiceTemplateService>>,
    template_id: Path<Uuid>,
    tenant_id: TenantId,
    _user_id: UserId,
) -> actix_web::Result<impl Responder> {
    let template = data.get_template_by_id(template_id.into_inner(),
                                           tenant_id.inner())
        .await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(template))
}

setup_routes!(InvoiceTemplateService,
    "/invoice-template",
    "/create",web::post().to(create_invoice_template),
    "/id/{address_id}",
    web::get().to(get_invoice_template_by_id));
#[cfg(test)]
mod tests{
    use uuid::Uuid;
    use crate::accounting::currency::currency_models::tests::an_audit_metadata_base;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;
    use crate::get_and_create_api_test_v2;
    use crate::invoicing::invoice_template::invoice_template_models::{CreateInvoiceTemplateRequest, InvoiceTemplateMaster};
    use crate::invoicing::invoice_template::invoice_template_service::{InvoiceTemplateService, MockInvoiceTemplateService};
    use crate::masters::company_master::company_master_models::base_master_fields::tests::a_base_master_field;
    use super::*;
    #[tokio::test]
    async fn test_create_invoice_template(){
        let expected_invoice_template:InvoiceTemplateMaster = InvoiceTemplateMaster{
            base_master_fields: a_base_master_field(Default::default()),
            sample_doc_s3_id: None,
            audit_metadata: an_audit_metadata_base(Default::default()),
        };
        let ak =Arc::new(expected_invoice_template.clone());
        let closure = move||{
            let mut mock:MockInvoiceTemplateService = MockInvoiceTemplateService::new();
            mock.expect_create_template()
                .returning(|_,_,_|Ok(Default::default()));
            mock.expect_get_template_by_id()
                .return_once(|_,_|Ok(Some(ak)));
            mock
        };
        let get_uri = format!("/invoice-template/id/{}",Uuid::default());
    
        let create_req = CreateInvoiceTemplateRequest{
            idempotence_key:Uuid::default(),
            sample_doc_s3_id:None
        };
        get_and_create_api_test_v2!(
          InvoiceTemplateMaster,
          InvoiceTemplateService,
          closure,
          get_uri,
          "/invoice-template/create",
          create_req,
          expected_invoice_template,
          *SEED_TENANT_ID
        );
    }
}