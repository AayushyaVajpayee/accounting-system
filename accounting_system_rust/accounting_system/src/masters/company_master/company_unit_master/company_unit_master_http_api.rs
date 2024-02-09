use actix_web::{HttpResponseBuilder, Responder, ResponseError, web};
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path, Query};
use std::sync::Arc;
use uuid::Uuid;

use crate::common_utils::pagination::pagination_utils::{
    PaginationRequest, set_pagination_headers,
};
use crate::masters::company_master::company_unit_master::company_unit_models::CreateCompanyUnitRequest;
use crate::masters::company_master::company_unit_master::company_unit_service::{
    CompanyUnitService, CompanyUnitServiceError,
};
use crate::setup_routes;

impl ResponseError for CompanyUnitServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            CompanyUnitServiceError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CompanyUnitServiceError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

async fn create_company_unit(
    data: Data<Arc<dyn CompanyUnitService>>,
    request: web::Json<CreateCompanyUnitRequest>,
) -> actix_web::Result<impl Responder> {
    let resp = data.create_company_unit(&request).await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(resp))
}

async fn get_company_unit_by_id(
    data: Data<Arc<dyn CompanyUnitService>>,
    company_unit_id: Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let resp = data
        .get_company_unit_by_id(&company_unit_id.into_inner())
        .await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).json(resp))
}

async fn get_company_units_by_company_id(
    data: Data<Arc<dyn CompanyUnitService>>,
    query: Query<PaginationRequest>,
    company_id: Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let resp = data
        .get_company_units_by_company_id(&company_id.into_inner(), &query.0)
        .await?;
    let mut response = HttpResponseBuilder::new(StatusCode::OK).json(&resp);
    let headers = response.headers_mut();
    set_pagination_headers(headers, &resp.meta);
    Ok(response)
}

setup_routes!(
    CompanyUnitService,
    "/company-unit-master",
    "/create",
    web::post().to(create_company_unit),
    "/company-unit-id/{company_unit_id}",
    web::get().to(get_company_unit_by_id),
    "/company-id/{company_id}",
    web::get().to(get_company_units_by_company_id)
);

#[cfg(test)]
mod tests {
    use actix_web::{App, test};
    use actix_web_lab::middleware::from_fn;
    use std::sync::Arc;
    use uuid::Uuid;

    use crate::common_utils::pagination::constants::{CURRENT_PAGE, LINKS, PER_PAGE, TOTAL_COUNT, TOTAL_PAGES};
    use crate::common_utils::pagination::pagination_utils::{PaginatedResponse, pagination_header_middleware, PaginationMetadata};
    use crate::get_and_create_api_test;
    use crate::masters::company_master::company_master_models::company_master::tests::SEED_COMPANY_MASTER_ID;
    use crate::masters::company_master::company_unit_master::company_unit_master_http_api::map_endpoints_to_functions;
    use crate::masters::company_master::company_unit_master::company_unit_models::{CompanyUnitMaster, CreateCompanyUnitRequestBuilder};
    use crate::masters::company_master::company_unit_master::company_unit_models::tests::a_create_company_unit_request;
    use crate::masters::company_master::company_unit_master::company_unit_service::{
        CompanyUnitService, MockCompanyUnitService,
    };

    #[tokio::test]
    async fn test_get_company_units_by_company_id() {
        let mut mocked = MockCompanyUnitService::new();
        mocked.expect_get_company_units_by_company_id()
            .returning(|_, data| Ok(PaginatedResponse::<CompanyUnitMaster> {
                data: vec![],
                meta: PaginationMetadata {
                    current_page: data.page_no,
                    page_size: data.per_page,
                    total_pages: 51,
                    total_count: 510,
                },
            }));
        let uri = format!("/company-unit-master/company-id/{}?page_no={}&per_page={}"
                          , *SEED_COMPANY_MASTER_ID, 2, 12);
        let mock: Arc<dyn CompanyUnitService> = Arc::new(mocked);
        let app_data = actix_web::web::Data::new(mock);
        let app = App::new()
            .wrap(from_fn(pagination_header_middleware))
            .service(map_endpoints_to_functions())
            .app_data(app_data);
        let app_service = test::init_service(app).await;
        let request = test::TestRequest::get().uri(&uri).to_request();
        let j = test::call_service(&app_service, request).await;
        assert!(j.headers().contains_key(TOTAL_COUNT));
        assert!(j.headers().contains_key(PER_PAGE));
        assert!(j.headers().contains_key(CURRENT_PAGE));
        assert!(j.headers().contains_key(TOTAL_PAGES));
        assert!(j.headers().contains_key(LINKS));
    }
    #[tokio::test]
    async fn test_create_company_unit_api() {
        let closure = || {
            let mut mocked = MockCompanyUnitService::new();
            mocked
                .expect_create_company_unit()
                .returning(|_| Ok(Default::default()));
            mocked
                .expect_get_company_unit_by_id()
                .returning(|_| Ok(Some(Default::default())));
            mocked
        };
        let get_uri = format!("/company-unit-master/company-unit-id/{}",Uuid::default());
        let expected_company_unit:CompanyUnitMaster=Default::default();
        let create_req =a_create_company_unit_request(CreateCompanyUnitRequestBuilder::default());
        get_and_create_api_test!(
            CompanyUnitMaster,
            CompanyUnitService,
            closure,get_uri,
            "/company-unit-master/create",
            create_req,
            expected_company_unit);
    }
}
