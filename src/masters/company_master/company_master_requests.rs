#[derive(Debug)]
pub struct CreateCompanyRequest{
    pub tenant_id:i32,
    pub name:String,
    pub cin:String,
    pub created_by:String,
}
