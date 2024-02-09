use anyhow::Context;
use anyhow::Error as AnyhowError;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use itertools::Itertools;
#[cfg(test)]
use mockall::automock;
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, instrument};
use uuid::Uuid;
use validator::Validate;

use crate::accounting::user::user_service::{UserService, UserServiceError};
use crate::common_utils::dao_error::DaoError;
use crate::common_utils::pagination::pagination_utils::{PaginatedResponse, PaginationRequest};
use crate::common_utils::utils::flatten_errors;
use crate::masters::company_master::company_master_models::company_identification_number::CompanyIdentificationNumber;
use crate::masters::company_master::company_master_models::company_master::CompanyMaster;
use crate::masters::company_master::company_master_models::company_name::CompanyName;
use crate::masters::company_master::company_master_request_response::CreateCompanyRequest;
use crate::masters::company_master::dao::dao_trait::CompanyMasterDao;
use crate::masters::company_master::dao::dao_trait_impl::get_company_master_dao;
use crate::tenant::tenant_service::{TenantService, TenantServiceError};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CompanyMasterService: Send + Sync {
    async fn get_all_companies_for_tenant_id(&self, tenant_id: &Uuid, pagination_request: &PaginationRequest) -> Result<PaginatedResponse<CompanyMaster>, ServiceError>;

    async fn create_new_company_for_tenant(
        &self,
        request: &CreateCompanyRequest,
    ) -> Result<Uuid, ServiceError>;

}

pub struct CompanyMasterServiceImpl {
    dao: Arc<dyn CompanyMasterDao>,
    tenant_service: Arc<dyn TenantService>,
    user_service: Arc<dyn UserService>,
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("validation failures \n {}", .0.join("\n"))]
    Validation(Vec<String>),
    //4xx
    #[error("error in db {}", 0)]
    Db(DaoError),
    //5xx
    //have to separate out idempotency check
    #[error("company with this cin already exists")]
    CompanyCinAlreadyExists,
    // company_master_pkey
    #[error("company with this is already exists")]
    CompanyWithPrimaryKeyExists,

    #[error(transparent)]
    Tenant(#[from] TenantServiceError),
    #[error(transparent)]
    UserService(#[from] UserServiceError),
    #[error(transparent)]
    AnyhowError(#[from] AnyhowError),
}

impl From<DaoError> for ServiceError {
    fn from(dao_err: DaoError) -> Self {
        match dao_err {
            DaoError::ConnectionPool(_) |
            DaoError::PostgresQueryError(_) |
            DaoError::InvalidEntityToDbRowConversion(_) | DaoError::ReturnedValueNone => ServiceError::Db(dao_err),
            DaoError::UniqueConstraintViolated {
                ref constraint_name,
            } => {
                if constraint_name.as_str() == "unique_cin_company" {
                    return ServiceError::CompanyCinAlreadyExists;
                } else if constraint_name.as_str() == "company_master_pkey" {
                    return ServiceError::CompanyWithPrimaryKeyExists;
                }
                ServiceError::Db(dao_err)
            }
            DaoError::AnyhowError(_) => {
                ServiceError::Db(dao_err)
            }
        }
    }
}

impl CompanyMasterServiceImpl {
    async fn validate_create_company_request(&self, request: &CreateCompanyRequest) -> Result<Vec<String>, ServiceError> {
        let mut validations = Vec::new();
        let tenant = self
            .tenant_service
            .get_tenant_by_id(request.tenant_id)
            .await?;
        let user = self.user_service.get_user_by_id(request.created_by).await?;
        let company_name = CompanyName::validate(request.name.as_str());
        let cin = CompanyIdentificationNumber::validate(request.cin.as_str());
        if tenant.is_none() {
            validations.push(format!("no tenant found for id {}", request.tenant_id))
        };
        if user.is_none() {
            validations.push(format!(
                "no user found for id in  created by {}",
                request.created_by
            ))
        };
        if let Err(e) = company_name {
            validations.push(e.to_string())
        };

        if let Err(e) = cin {
            validations.push(e.to_string());
        };
        Ok(validations)
    }
}

#[async_trait]
impl CompanyMasterService for CompanyMasterServiceImpl {
    async fn get_all_companies_for_tenant_id(&self, tenant_id: &Uuid, pagination_request: &PaginationRequest) -> Result<PaginatedResponse<CompanyMaster>, ServiceError> {
        let validated = pagination_request.validate();
        if let Err(validated) = validated {
            let errs = flatten_errors(&validated)
                .context("flatten_errors failed in CompanyMasterService.get_all_companies_for_tenant_id")?;
            let errs = errs.iter().map(|a| a.to_string()).collect_vec();
            return Err(ServiceError::Validation(errs));
        }
        let resp = self.dao.get_all_companies_for_tenant(tenant_id, pagination_request.page_no, pagination_request.per_page).await?;
        return Ok(resp);
    }

    #[instrument(skip(self))]
    async fn create_new_company_for_tenant(
        &self,
        request: &CreateCompanyRequest,
    ) -> Result<Uuid, ServiceError> {
        let validations = self.validate_create_company_request(request).await?;
        if !validations.is_empty() {
            return Err(ServiceError::Validation(validations));
        }
        let company_master = request.to_company_master()
            .context("error while converting company creation request to company master")?;
        let res = self
            .dao
            .create_new_company_for_tenant(&company_master, &request.idempotence_key)
            .await?;
        Ok(res)
    }
}

pub fn get_company_master_service(arc: Arc<Pool>, tenant_service: Arc<dyn TenantService>, user_service: Arc<dyn UserService>) -> Arc<dyn CompanyMasterService> {
    let dao = get_company_master_dao(arc);
    Arc::new(CompanyMasterServiceImpl { dao, tenant_service, user_service })
}

#[cfg(test)]
pub mod tests {
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::{ResultAssertions, VecAssertions};
    use std::mem::discriminant;
    use std::sync::Arc;
    use tracing_test::traced_test;

    use crate::accounting::postgres_factory::test_utils_postgres::{
        get_postgres_conn_pool, get_postgres_image_port,
    };
    use crate::accounting::user::user_models::tests::a_user;
    use crate::accounting::user::user_service::{
        get_user_service_for_test, MockUserService, UserService,
    };
    use crate::common_utils::dao_error::DaoError;
    use crate::masters::company_master::company_master_request_response::tests::{
        a_create_company_request, CreateCompanyRequestBuilder,
    };
    use crate::masters::company_master::company_master_service::{
        CompanyMasterService, CompanyMasterServiceImpl, ServiceError,
    };
    use crate::masters::company_master::dao::dao_trait::MockCompanyMasterDao;
    use crate::masters::company_master::dao::dao_trait_impl::get_company_master_dao;
    use crate::tenant::tenant_models::tests::a_tenant;
    use crate::tenant::tenant_service::{get_tenant_service, MockTenantService, TenantService};

    pub async fn get_company_master_service_for_tests() -> Arc<dyn CompanyMasterService> {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let tenant_service = get_tenant_service(postgres_client.clone());
        let user_service = get_user_service_for_test(postgres_client.clone());
        let dao = get_company_master_dao(postgres_client.clone());
        let service = CompanyMasterServiceImpl {
            dao,
            tenant_service,
            user_service,
        };
        Arc::new(service)
    }

    #[traced_test]
    #[rstest]
    #[case("unique_cin_company", ServiceError::CompanyCinAlreadyExists)]
    #[case("company_master_pkey", ServiceError::CompanyWithPrimaryKeyExists)]
    #[case("some_random_constraint", ServiceError::Db(DaoError::UniqueConstraintViolated{constraint_name: "some_random_constraint".to_string()}))]
    async fn test_duplicate_cin_insertion(
        #[case] constraint_name: String,
        #[case] error: ServiceError,
    ) {
        // tracing_subscriber::fmt::fmt()
        //     .pretty()
        //     .with_span_events(FmtSpan::FULL)
        //     .with_max_level(tracing::Level::INFO)
        //     .init();
        let mut dao = MockCompanyMasterDao::new();
        let mut user_service = MockUserService::new();
        let mut tenant_service = MockTenantService::new();
        dao.expect_create_new_company_for_tenant()
            .return_once(|_a, _| Err(DaoError::UniqueConstraintViolated { constraint_name }))
            .once();
        tenant_service
            .expect_get_tenant_by_id()
            .returning(|_a| Ok(Some(a_tenant(Default::default()))))
            .once();
        user_service
            .expect_get_user_by_id()
            .returning(|_a| Ok(Some(a_user(Default::default()))))
            .once();
        let  company_service = CompanyMasterServiceImpl {
            dao: Arc::new(dao),
            tenant_service: Arc::new(tenant_service),
            user_service: Arc::new(user_service),
        };
        let company_request = a_create_company_request(Default::default());
        let output = company_service
            .create_new_company_for_tenant(&company_request)
            .await;
        let expected_err = discriminant(&error);
        let actual_err = discriminant(output.as_ref().unwrap_err());
        assert_that!(output).is_err();
        assert_that!(actual_err).is_equal_to(expected_err);
    }


    #[tokio::test]
    async fn test_user_not_found_validation() {
        let mut user_service = MockUserService::new();
        let mut tenant_service = MockTenantService::new();
        user_service.expect_get_user_by_id().returning(|_a| Ok(None));
        tenant_service
            .expect_get_tenant_by_id()
            .returning(|a| Ok(Some(a_tenant(Default::default()))));
        let company_request = a_create_company_request(Default::default());
        let user_service: Arc<dyn UserService> = Arc::new(user_service);
        let tenant_service: Arc<dyn TenantService> = Arc::new(tenant_service);
        let company_master_service = CompanyMasterServiceImpl {
            dao: Arc::new(MockCompanyMasterDao::new()),
            user_service,
            tenant_service,
        };
        let errors = company_master_service
            .validate_create_company_request(&company_request)
            .await.unwrap();
        assert_that!(errors).has_length(1);
        let error = errors.get(0).unwrap();
        let p = format!(
            "no user found for id in  created by {}",
            company_request.created_by
        );
        assert_that!(error).is_equal_to(&p);
    }

    #[tokio::test]
    async fn test_tenant_not_found_validation() {
        let mut user_service = MockUserService::new();
        let mut tenant_service = MockTenantService::new();
        user_service
            .expect_get_user_by_id()
            .returning(|_a| Ok(Some(a_user(Default::default()))));
        tenant_service
            .expect_get_tenant_by_id()
            .returning(|_a| Ok(None));
        let company_request = a_create_company_request(Default::default());
        let user_service: Arc<dyn UserService> = Arc::new(user_service);
        let tenant_service: Arc<dyn TenantService> = Arc::new(tenant_service);
        let company_master_service = CompanyMasterServiceImpl {
            dao: Arc::new(MockCompanyMasterDao::new()),
            user_service,
            tenant_service,
        };
        let errors = company_master_service
            .validate_create_company_request(&company_request)
            .await.unwrap();
        assert_that!(errors).has_length(1);
        let error = errors.get(0).unwrap();
        let p = format!("no tenant found for id {}", company_request.tenant_id);
        assert_that!(error).is_equal_to(&p);
    }

    #[rstest]
    #[case(None, Some("flajkdjalfkjadlddkfjalkjflkajfljasdlfjdsalkjfdlajfd".to_string()), "company name cannot be empty or more than 50 chars")]
    #[case(Some("ljljljlkjlkjlkjljljlkjlj".to_string()), None, "cin length should be 21 chars and should be alphanumeric")]
    async fn test_company_name_incorrect_validation(
        #[case] cin: Option<String>,
        #[case] name: Option<String>,
        #[case] error_message: String,
    ) {
        let mut user_service = MockUserService::new();
        let mut tenant_service = MockTenantService::new();
        user_service
            .expect_get_user_by_id()
            .returning(|_a| Ok(Some(a_user(Default::default()))));
        tenant_service
            .expect_get_tenant_by_id()
            .returning(|_a| Ok(Some(a_tenant(Default::default()))));
        let company_request = a_create_company_request(CreateCompanyRequestBuilder {
            name,
            cin,
            ..Default::default()
        });

        let user_service: Arc<dyn UserService> = Arc::new(user_service);
        let tenant_service: Arc<dyn TenantService> = Arc::new(tenant_service);
        let company_master_service = CompanyMasterServiceImpl {
            dao: Arc::new(MockCompanyMasterDao::new()),
            user_service,
            tenant_service,
        };
        let errors = company_master_service
            .validate_create_company_request(&company_request)
            .await.unwrap();
        assert_that!(errors).has_length(1);
        let error = errors.get(0).unwrap();
        assert_that!(error.as_str()).is_equal_to(error_message.as_str());
    }
}
