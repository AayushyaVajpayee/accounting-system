use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyMasterSql {
    pub id: uuid::Uuid,
    pub entity_version_id: i32,
    pub tenant_id: uuid::Uuid,
    pub active: bool,
    pub approval_status: i16,
    pub remarks: Option<String>,
    pub name: String,
    pub cin: String,
    pub created_by: uuid::Uuid,
    pub updated_by: uuid::Uuid,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedDbResponse<T> {
    pub rows: Vec<T>,
    pub total_pages: u32,
    pub total_count: u32,
}