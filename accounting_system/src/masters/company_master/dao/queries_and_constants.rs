use const_format::concatcp;
use crate::masters::company_master::company_master_models::master_status_enum::MasterStatusEnum;

pub const SELECT_FIELDS: &str = "id,entity_version_id,tenant_id,active,approval_status,remarks,name,cin,created_by,updated_by,created_at,updated_at";
pub const TABLE_NAME: &str = "company_master";

pub const GET_BY_ID: &str = concatcp!(
    "select ",
    SELECT_FIELDS,
    " from ",
    TABLE_NAME,
    " where id=$1 and tenant_id=$2 and approval_status=",
    MasterStatusEnum::Approved as i32
);
pub const GET_ALL_FOR_TENANT: &str = "select get_paginated_data($1,$2,$3,$4)";

pub const SOFT_DELETE: &str = concatcp!(
    "update ",
    TABLE_NAME,
    " set approval_status=",
    MasterStatusEnum::Deleted as i32,
    " ,entity_version_id=entity_version_id+1,remarks=$4,updated_by=$5,updated_at=extract(epoch from now()) * 1000000",
    " where id=$1 and tenant_id=$2 and entity_version_id=$3"
);
