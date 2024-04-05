use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponseBuilder, Responder, ResponseError};
use uuid::Uuid;

use crate::common_utils::utils::{TenantId, UserId};
use crate::masters::product_item_master::product_item_models::ProductCreationRequest;
use crate::masters::product_item_master::product_item_service::{
    ProductItemService, ProductItemServiceError,
};
use crate::setup_routes;

impl ResponseError for ProductItemServiceError {}

async fn create_product_item(
    data: Data<Arc<dyn ProductItemService>>,
    request: web::Json<ProductCreationRequest>,
    tenant_id: TenantId,
    user_id: UserId,
) -> actix_web::Result<impl Responder> {
    let ap = data
        .create_product(request.0, tenant_id.inner(), user_id.inner())
        .await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(ap))
}

async fn get_product_item(
    data: Data<Arc<dyn ProductItemService>>,
    product_id: web::Path<Uuid>,
    _tenant_id: TenantId,
    _user_id: UserId,
) -> actix_web::Result<impl Responder> {
    let ap = data
        .get_product(product_id.into_inner(), _tenant_id.inner())
        .await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(ap))
}

setup_routes!(
    ProductItemService,
    "/product-item",
    "/create",
    web::post().to(create_product_item),
    "/id/{id}",
    web::get().to(get_product_item)
);

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::get_and_create_api_test_v2;
    use crate::masters::product_item_master::product_item_models::tests::{
        a_product_creation_request, a_product_item_response,
    };
    use crate::masters::product_item_master::product_item_models::ProductItemResponse;
    use crate::masters::product_item_master::product_item_service::{
        MockProductItemService, ProductItemService,
    };
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    use super::map_endpoints_to_functions;

    #[tokio::test]
    async fn test_create_product_item() {
        let request = a_product_creation_request(Default::default());
        let response = a_product_item_response(Default::default());
        let cloned_resp = response.clone();
        let closure = move || {
            let mut product_service: MockProductItemService = MockProductItemService::new();
            product_service
                .expect_create_product()
                .returning(|_, _, _| Ok(Default::default()));
            product_service
                .expect_get_product()
                .return_once(|_, _| Ok(Some(Arc::new(cloned_resp))));
            product_service
        };
        let id = Uuid::now_v7();
        let get_uri = format!("/product-item/id/{id}");
        get_and_create_api_test_v2!(
            ProductItemResponse,
            ProductItemService,
            closure,
            get_uri,
            "/product-item/create",
            request,
            response,
            *SEED_TENANT_ID
        );
    }
}
