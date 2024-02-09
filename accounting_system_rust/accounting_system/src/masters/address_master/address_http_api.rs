use actix_web::{HttpResponseBuilder, Responder, ResponseError, web};
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use std::sync::Arc;
use uuid::Uuid;

use crate::masters::address_master::address_model::CreateAddressRequest;
use crate::masters::address_master::address_service::{AddressService, AddressServiceError};
use crate::setup_routes;

impl ResponseError for AddressServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            AddressServiceError::Db(_) => { StatusCode::INTERNAL_SERVER_ERROR }
        }
    }
}


async fn create_address(data: Data<Arc<dyn AddressService>>, request: web::Json<CreateAddressRequest>) -> actix_web::Result<impl Responder> {
    let ap = data.create_address(&request).await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(ap))
}

async fn get_address_by_id(data: Data<Arc<dyn AddressService>>, address_id: Path<Uuid>) -> actix_web::Result<impl Responder> {
    let ap = data.get_address_by_id(&address_id).await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(ap))
}

setup_routes!(AddressService,"/address",
    "/create",web::post().to(create_address),
"/id/{address_id}",web::get().to(get_address_by_id));


#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::get_and_create_api_test;
    use crate::masters::address_master::address_http_api::map_endpoints_to_functions;
    use crate::masters::address_master::address_model::{Address, CreateAddressRequestBuilder};
    use crate::masters::address_master::address_model::tests::a_create_address_request;
    use crate::masters::address_master::address_service::{AddressService, MockAddressService};

    #[tokio::test]
    async fn test_create_address() {
        let closure = || {
            let mut mock = MockAddressService::new();
            mock.expect_create_address()
                .returning(|_| Ok(Default::default()));
            mock.expect_get_address_by_id()
                .returning(|_| Ok(Some(Default::default())));
            mock
        };
        let get_uri = format!("/address/id/{}", Uuid::default());
        let expected_address: Address = Default::default();
        get_and_create_api_test!(Address,AddressService,closure,
            get_uri,
            "/address/create",
            a_create_address_request(CreateAddressRequestBuilder::default()),
            expected_address);
    }
}