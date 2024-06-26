use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{web, HttpResponseBuilder, Responder, ResponseError};
use uuid::Uuid;

use crate::common_utils::utils::{TenantId, UserId};
use crate::masters::business_entity_master::business_entity_models::{CreateBusinessEntityRequestRaw};
use crate::masters::business_entity_master::business_entity_service::{
    BusinessEntityService, BusinessEntityServiceError,
};
use crate::setup_routes;

impl ResponseError for BusinessEntityServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            BusinessEntityServiceError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
            BusinessEntityServiceError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

async fn create_business_entity_master(
    data: Data<Arc<dyn BusinessEntityService>>,
    request: web::Json<CreateBusinessEntityRequestRaw>,
    tenant_id: TenantId,
    user_id: UserId,
) -> actix_web::Result<impl Responder> {
    let ap = data
        .create_business_entity(request.into_inner(), tenant_id.inner(), user_id.inner())
        .await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(ap))
}

async fn get_business_entity_master_by_id(
    data: Data<Arc<dyn BusinessEntityService>>,
    business_entity_id: Path<Uuid>,
    tenant_id: TenantId,
) -> actix_web::Result<impl Responder> {
    let pd = data
        .get_business_entity_by_id(&business_entity_id, &tenant_id.inner())
        .await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(pd))
}
setup_routes!(
    BusinessEntityService,
    "/business-entity",
    "/create",
    web::post().to(create_business_entity_master),
    "/id/{business_entity_id}",
    web::get().to(get_business_entity_master_by_id)
);

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::get_and_create_api_test_v2;
    use crate::masters::business_entity_master::business_entity_http_api::map_endpoints_to_functions;
    use crate::masters::business_entity_master::business_entity_models::tests::a_create_business_entity_request;
    use crate::masters::business_entity_master::business_entity_models::BusinessEntityDto;
    use crate::masters::business_entity_master::business_entity_service::{
        BusinessEntityService, MockBusinessEntityService,
    };
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    #[tokio::test]
    async fn test_create_and_get_business_entity() {
        let closure = move || {
            let mut mock = MockBusinessEntityService::new();
            mock.expect_get_business_entity_by_id()
                .returning(move |_, _| Ok(Some(Default::default())));
            mock.expect_create_business_entity()
                .returning(|_, _, _| Ok(Default::default()));
            mock
        };
        let p: BusinessEntityDto = Default::default();
        let get_uri = format!("/business-entity/id/{}", Uuid::default());
        get_and_create_api_test_v2!(
            BusinessEntityDto,
            BusinessEntityService,
            closure,
            get_uri,
            "/business-entity/create",
            a_create_business_entity_request(Default::default()),
            p,
            *SEED_TENANT_ID
        );
    }
}
