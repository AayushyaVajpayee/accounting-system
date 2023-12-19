use anyhow::Context;
use tokio_postgres::Row;
use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;
use crate::masters::company_master::company_master_models::master_status_enum::MasterStatusEnum;
use crate::masters::company_master::company_master_models::master_updation_remarks::MasterUpdationRemarks;

pub fn convert_row_to_base_master_fields(row: &Row) -> anyhow::Result<(BaseMasterFields, usize)> {
    let remarks: Option<&str> = row.get(5);
    let remarks = if remarks.is_some() {
        let k = MasterUpdationRemarks::new(remarks.unwrap())?;
        Some(k)
    } else {
        None
    };
    let k = BaseMasterFields {
        id: row.try_get(0)?,
        entity_version_id: row.try_get(1)?,
        tenant_id: row.try_get(2)?,
        active: row.try_get(3)?,
        approval_status: MasterStatusEnum::get_enum_for_value(
            row.get::<usize, i16>(4) as usize
        )?,
        remarks,
    };
    Ok((k, 6))
}

pub fn convert_row_to_audit_metadata_base(start_ind: usize, row: &Row) -> anyhow::Result<AuditMetadataBase> {
    let a = AuditMetadataBase {
        created_by: row.try_get(start_ind)?,
        updated_by: row.try_get(start_ind + 1)?,
        created_at: row.try_get(start_ind + 2)?,
        updated_at: row.try_get(start_ind + 3)?,
    };
    Ok(a)
}