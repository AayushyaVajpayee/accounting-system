use crate::masters::company_master::company_master_dao::CompanyMasterDao;
use crate::masters::company_master::company_master_requests::CreateCompanyRequest;
use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait CompanyMasterService {
    // async fn get_all_companies_for_tenant_id(&self,tenant_id:i32);

    // async fn get_all_company_units_for_company_id_and_tenant_id(&self,tenant_id:i32);

    async fn create_new_company_for_tenant(&self, request: &CreateCompanyRequest);

    // async fn create_new_company_unit_for_tenant_and_company_id(&self,tenant_id:i32);
    //
    // async fn soft_delete_company(&self);
}

pub struct CompanyMasterServiceImpl {
    dao: Box<dyn CompanyMasterDao + Send + Sync>,
}

#[derive(Debug, Error)]
pub enum ServiceErrors {
    #[error("j")]
    ValidationError(String), //4xx
    #[error("jd")]
    DBError, //5xx
             //have to separate out idempotency check
}

#[async_trait]
impl CompanyMasterService for CompanyMasterServiceImpl {
    async fn create_new_company_for_tenant(&self, request: &CreateCompanyRequest) {
        //1. validate request
        //2. check for idempotency without hitting database if possible currently not possible i think without db
        //3. if already processed then return
        //4. if not processed then hit the db with the created object
        //5. check for any errors and transform them accordingly
        // do step 2,3,4 step in the database for better efficiency
        // self.dao.create_new_company_for_tenant()
        //todo validations
        //tenant_id should be valid
        //cin should be alphanumeric and 21 chars
        //name should be 50 chars max
        //createdBy should be non empty and a valid user I think
    }
}
