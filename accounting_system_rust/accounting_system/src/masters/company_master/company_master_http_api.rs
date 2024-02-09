use actix_web::{HttpResponse, HttpResponseBuilder, Responder, web};
use actix_web::body::BoxBody;
use actix_web::error::ResponseError;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path, Query};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::Arc;
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::common_utils::pagination::pagination_utils::{PaginationRequest, set_pagination_headers};
use crate::masters::company_master::company_master_request_response::CreateCompanyRequest;
use crate::masters::company_master::company_master_service::{CompanyMasterService, ServiceError};
use crate::setup_routes;

async fn get_companies_by_tenant_id(data: Data<Arc<dyn CompanyMasterService>>, query: Query<PaginationRequest>, tenant_id: Path<Uuid>) -> actix_web::Result<impl Responder> {
    let resp = data.get_all_companies_for_tenant_id(&tenant_id, &query.0).await?;
    let mut response = HttpResponseBuilder::new(StatusCode::OK).json(&resp);
    let headers = response.headers_mut();
    set_pagination_headers(headers, &resp.meta);
    Ok(response)
}

#[instrument(skip(data))]
pub async fn create_company(
    request: web::Json<CreateCompanyRequest>,
    data: Data<Arc<dyn CompanyMasterService>>,
) -> actix_web::Result<impl Responder> {
    debug!(?request, "create company request received");
    let created_uuid = data.create_new_company_for_tenant(request.deref()).await?;
    let response = HttpResponseBuilder::new(StatusCode::CREATED).json(created_uuid);
    Ok(response)
}

#[derive(Serialize, Debug)]
struct Errors<'a> {
    errors: &'a Vec<String>,
}

#[derive(Deserialize, Debug)]
struct ErrorsResponse {
    errors: Vec<String>,
}

impl ResponseError for ServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::Validation(_) => StatusCode::BAD_REQUEST,
            ServiceError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::CompanyCinAlreadyExists => StatusCode::CONFLICT,
            ServiceError::CompanyWithPrimaryKeyExists => StatusCode::CONFLICT,
            ServiceError::AnyhowError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::Tenant(err) => { err.status_code() }
            ServiceError::UserService(err) => { err.status_code() }
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            ServiceError::Validation(errs) => {
                HttpResponse::build(self.status_code()).json(Errors { errors: errs })
            }
            ServiceError::Db(errs) => {
                let err_list = vec![errs.to_string()];
                HttpResponse::build(self.status_code()).json(Errors { errors: &err_list })
            }
            ServiceError::CompanyCinAlreadyExists => {
                let err_list = vec![self.to_string()];
                HttpResponse::build(self.status_code()).json(Errors { errors: &err_list })
            }
            ServiceError::CompanyWithPrimaryKeyExists => {
                let err_list = vec![self.to_string()];
                HttpResponse::build(self.status_code()).json(Errors { errors: &err_list })
            }
            ServiceError::AnyhowError(_) => {
                let err_list = vec![self.to_string()];
                HttpResponse::build(self.status_code()).json(Errors { errors: &err_list })
            }
            ServiceError::Tenant(err) => { err.error_response() }
            ServiceError::UserService(err) => { err.error_response() }
        }
    }
}


setup_routes!(CompanyMasterService,"/company-master",
    "/tenant-id/{tenant_id}",web::get().to(get_companies_by_tenant_id),
    "/create",web::post().to(create_company));


#[cfg(test)]
mod tests {
    use actix_web::{App, test};
    use actix_web::middleware::Logger;
    use anyhow::anyhow;
    use bytes::Buf;
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::VecAssertions;
    use std::sync::Arc;
    use tracing::info;
    use tracing_test::traced_test;
    use uuid::Uuid;

    use crate::common_utils::dao_error::DaoError;
    use crate::masters::company_master::company_master_http_api::{ErrorsResponse, map_endpoints_to_functions};
    use crate::masters::company_master::company_master_request_response::tests::a_create_company_request;
    use crate::masters::company_master::company_master_service::{CompanyMasterService, MockCompanyMasterService, ServiceError};

    #[traced_test]
    #[tokio::test]
    async fn create_company_test() {
        // std::env::set_var(env"RUST_LOG", "debug");
        // env_logger::init();
        let mut mocked = MockCompanyMasterService::new();
        let uuid = Uuid::now_v7();
        mocked
            .expect_create_new_company_for_tenant()
            .returning(move |_a| Ok(uuid));
        let mock: Arc<dyn CompanyMasterService> = Arc::new(mocked);
        let app_data = actix_web::web::Data::new(mock);
        let app = App::new()
            .wrap(Logger::default())
            .service(map_endpoints_to_functions())
            .app_data(app_data);
        let app_service = test::init_service(app).await;
        let create_company_req = a_create_company_request(Default::default());
        info!(?create_company_req, "create company request created");

        let request = test::TestRequest::post()
            .uri("/company-master/create")
            .set_json(create_company_req)
            .to_request();
        let res: Uuid = test::call_and_read_body_json(&app_service, request).await;
        assert_eq!(res, uuid);
    }

    #[rstest]
    #[case(ServiceError::Validation(vec ! ["some rr".to_string()]), 400)]
    #[case(ServiceError::Db(DaoError::ConnectionPool("pool exhausted".to_string())), 500)]
    #[case(ServiceError::CompanyCinAlreadyExists, 409)]
    #[case(ServiceError::CompanyWithPrimaryKeyExists, 409)]
    #[case(ServiceError::AnyhowError(anyhow ! ("some other error")), 500)]
    async fn create_company_test_error(#[case] err: ServiceError, #[case] http_code: u16) {
        // std::env::set_var(env"RUST_LOG", "debug");
        // env_logger::init();
        let mut mocked = MockCompanyMasterService::new();
        let uuid = Uuid::now_v7();
        mocked
            .expect_create_new_company_for_tenant()
            .return_once(move |_a| Err(err));
        let mock: Arc<dyn CompanyMasterService> = Arc::new(mocked);
        let app_data = actix_web::web::Data::new(mock);
        let app = App::new()
            .wrap(Logger::default())
            .service(map_endpoints_to_functions())
            .app_data(app_data);
        let app_service = test::init_service(app).await;
        let create_company_req = a_create_company_request(Default::default());
        info!(?create_company_req, "create company request created");

        let request = test::TestRequest::post()
            .uri("/company-master/create")
            .set_json(create_company_req)
            .to_request();
        let res = test::call_service(&app_service, request).await;
        // let pdfda = res.reader();
        // let p = pdfda.to_string();
        assert_eq!(res.status().as_u16(), http_code);
        let p: ErrorsResponse = test::read_body_json(res).await;
        assert_that!(p.errors).has_length(1);
    }
}
