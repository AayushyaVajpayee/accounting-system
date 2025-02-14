use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{web, HttpResponseBuilder, Responder, ResponseError};
use uuid::Uuid;

use crate::common_utils::utils::{TenantId, UserId};
use crate::invoicing::invoicing_series::invoicing_series_models::CreateInvoiceNumberSeriesRequest;
use crate::invoicing::invoicing_series::invoicing_series_service::{
    InvoicingSeriesService, InvoicingSeriesServiceError,
};
use crate::setup_routes;

impl ResponseError for InvoicingSeriesServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            InvoicingSeriesServiceError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

async fn create_invoice_series(
    data: Data<Arc<dyn InvoicingSeriesService>>,
    request: web::Json<CreateInvoiceNumberSeriesRequest>,
    tenant_id: TenantId,
    user_id: UserId,
) -> actix_web::Result<impl Responder> {
    let ap = data
        .create_invoice_series(&request, tenant_id.inner(), user_id.inner())
        .await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(ap))
}

async fn get_invoice_series(
    data: Data<Arc<dyn InvoicingSeriesService>>,
    invoicing_series_id: Path<Uuid>,
    tenant_id: TenantId,
) -> actix_web::Result<impl Responder> {
    let ap = data
        .get_invoicing_series_by_id(invoicing_series_id.into_inner(), tenant_id.inner())
        .await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(ap))
}

setup_routes!(
    InvoicingSeriesService,
    "/invoice-no-series",
    "/create",
    web::post().to(create_invoice_series),
    "/id/{invoicing_series_id}",
    web::get().to(get_invoice_series)
);

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::get_and_create_api_test_v2;
    use crate::invoicing::invoicing_series::invoicing_series_http_api::map_endpoints_to_functions;
    use crate::invoicing::invoicing_series::invoicing_series_models::tests::a_create_invoice_number_series_request;
    use crate::invoicing::invoicing_series::invoicing_series_models::InvoicingSeriesMaster;
    use crate::invoicing::invoicing_series::invoicing_series_service::{
        InvoicingSeriesService, MockInvoicingSeriesService,
    };
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    #[tokio::test]
    async fn test_get_and_create_api() {
        let closure = || {
            let mut mock = MockInvoicingSeriesService::new();
            mock.expect_create_invoice_series()
                .returning(|_, _, _| Ok(Default::default()));
            mock.expect_get_invoicing_series_by_id()
                .returning(|_, _| Ok(Some(Default::default())));
            mock
        };
        let get_uri = format!("/invoice-no-series/id/{}", Uuid::default());
        let expected_val: InvoicingSeriesMaster = Default::default();
        get_and_create_api_test_v2!(
            InvoicingSeriesMaster,
            InvoicingSeriesService,
            closure,
            get_uri,
            "/invoice-no-series/create",
            a_create_invoice_number_series_request(Default::default()),
            expected_val,
            *SEED_TENANT_ID
        );
    }
}
