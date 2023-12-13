use async_trait::async_trait;

#[async_trait]
pub trait CompanyUnitDao {
    async fn create_company_unit();
    async fn get_company_unit_by_id();
    async fn get_company_units_by_company_id();
}