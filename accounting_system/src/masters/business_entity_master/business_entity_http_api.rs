use std::str::FromStr;
use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponseBuilder, Responder, ResponseError, web};
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use thiserror::Error;
use uuid::Uuid;

use crate::masters::business_entity_master::business_entity_models::CreateBusinessEntityRequest;
use crate::masters::business_entity_master::business_entity_service::{BusinessEntityService, BusinessEntityServiceError};
use crate::setup_routes;

impl ResponseError for BusinessEntityServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            BusinessEntityServiceError::Db(_) => { StatusCode::INTERNAL_SERVER_ERROR }
        }
    }
}

impl ResponseError for TenantHeaderError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[derive(Debug, Error)]
pub enum TenantHeaderError {
    #[error("x-tenant-id header not present in request")]
    NotPresent,
    #[error("x-tenant-id header does not have a valid uuid")]
    NotUuid,
}

async fn create_business_entity_master(data: Data<Arc<dyn BusinessEntityService>>, request: web::Json<CreateBusinessEntityRequest>) -> actix_web::Result<impl Responder> {
    let ap = data.create_business_entity(&request).await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(ap))
}

async fn get_business_entity_master_by_id(data: Data<Arc<dyn BusinessEntityService>>, business_entity_id: Path<Uuid>, req: HttpRequest) -> actix_web::Result<impl Responder> {
    let p = req.headers()
        .get("x-tenant-id")
        .ok_or(TenantHeaderError::NotPresent)?;
    let tenant_id_str = p.to_str().map_err(|a| TenantHeaderError::NotUuid)?;
    let tenant_uuid = Uuid::from_str(tenant_id_str).map_err(|a| TenantHeaderError::NotUuid)?;
    let pd = data.get_business_entity_by_id(&business_entity_id, &tenant_uuid).await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(pd))
}
setup_routes!(BusinessEntityService,"/business-entity",
    "/create",web::post().to(create_business_entity_master),
"/id/{business_entity_id}",web::get().to(get_business_entity_master_by_id));

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::get_and_create_api_test_v2;
    use crate::masters::business_entity_master::business_entity_http_api::map_endpoints_to_functions;
    use crate::masters::business_entity_master::business_entity_models::BusinessEntityMaster;
    use crate::masters::business_entity_master::business_entity_models::tests::a_create_business_entity_request;
    use crate::masters::business_entity_master::business_entity_service::{BusinessEntityService, MockBusinessEntityService};
    use crate::tenant::tenant_models::SEED_TENANT_ID;

    #[tokio::test]
    async fn test_create_and_get_business_entity() {
        let expected_val: BusinessEntityMaster = Default::default();
        let closure = move || {
            let mut mock = MockBusinessEntityService::new();
            mock.expect_get_business_entity_by_id()
                .returning(move |_, _| Ok(Some(Default::default())));
            mock.expect_create_business_entity()
                .returning(|_| Ok(Default::default()));
            mock
        };
        let get_uri = format!("/business-entity/id/{}", Uuid::default());
        get_and_create_api_test_v2!(BusinessEntityMaster,
            BusinessEntityService,
            closure,get_uri,
            "/business-entity/create",
            a_create_business_entity_request(Default::default()),
            expected_val,
            *SEED_TENANT_ID
        );
    }
}
