use std::fmt::Display;
use std::sync::Arc;

use actix_web::{HttpResponseBuilder, Responder, ResponseError, web};
use actix_web::http::StatusCode;
use actix_web::web::Data;

use crate::common_utils::utils::{TenantId, UserId};
use crate::invoicing::invoicing_request_models::{CreateInvoiceRequest, InvoicePdfRequest};
use crate::invoicing::invoicing_service::{InvoicingService, InvoicingServiceError};
use crate::setup_routes;

impl ResponseError for InvoicingServiceError{
    
}
async fn create_invoice(
    data:Data<Arc<dyn InvoicingService>>,
    request:web::Json<CreateInvoiceRequest>,
    tenant_id: TenantId,
    user_id: UserId
)-> actix_web::Result<impl Responder>{
    let ap = data
        .create_invoice(&request,tenant_id.inner(),user_id.inner())
        .await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(ap))
}

async fn create_invoice_pdf(
    data:Data<Arc<dyn InvoicingService>>,
    request:web::Json<InvoicePdfRequest>,
    _tenant_id: TenantId,
    _user_id: UserId
)-> actix_web::Result<impl Responder>{
    let ap = data.create_invoice_pdf(
        request.into_inner()
    ).await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(ap))
}



setup_routes!(InvoicingService,"/invoice",
    "/create",web::post().to(create_invoice),
    "/create-pdf",web::post().to(create_invoice_pdf));

