use anyhow::Context;
use tokio_postgres::Row;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::common_utils::dao_error::DaoError;
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;
use crate::masters::company_master::company_master_models::company_identification_number::CompanyIdentificationNumber;
use crate::masters::company_master::company_master_models::company_master::CompanyMaster;
use crate::masters::company_master::company_master_models::company_name::CompanyName;
use crate::masters::company_master::company_master_models::master_status_enum::MasterStatusEnum;
use crate::masters::company_master::company_master_models::master_updation_remarks::MasterUpdationRemarks;
use crate::masters::company_master::dao::models::CompanyMasterSql;

impl TryFrom<&Row> for CompanyMaster {
    type Error = DaoError;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        const DIAGNOSTIC_HELP: &str = "failed conversion of CompanyMasterSql to CompanyMaster";
        let remarks: Option<&str> = row.get(5);
        let remarks = if remarks.is_some() {
            let k = MasterUpdationRemarks::new(remarks.unwrap()).context(DIAGNOSTIC_HELP)?;
            Some(k)
        } else {
            None
        };
        Ok(CompanyMaster {
            base_master_fields: BaseMasterFields {
                id: row.get(0),
                entity_version_id: row.get(1),
                tenant_id: row.get(2),
                active: row.get(3),
                approval_status: MasterStatusEnum::get_enum_for_value(
                    row.get::<usize, i16>(4) as usize
                ).context(DIAGNOSTIC_HELP)?,
                remarks,
            },
            name: CompanyName::new(row.get(6)).context(DIAGNOSTIC_HELP)?,
            cin: CompanyIdentificationNumber::new(row.get(7)).context(DIAGNOSTIC_HELP)?,
            audit_metadata: AuditMetadataBase {
                created_by: row.get(8),
                updated_by: row.get(9),
                created_at: row.get(10),
                updated_at: row.get(11),
            },
        })
    }
}


impl TryFrom<Row> for CompanyMasterSql {
    type Error = DaoError;
    fn try_from(row: Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.try_get(0)?,
            entity_version_id: row.try_get(1)?,
            tenant_id: row.try_get(2)?,
            active: row.try_get(3)?,
            approval_status: row.try_get(4)?,
            remarks: row.try_get(5)?,
            name: row.try_get(6)?,
            cin: row.try_get(7)?,
            created_by: row.try_get(8)?,
            updated_by: row.try_get(9)?,
            created_at: row.try_get(10)?,
            updated_at: row.try_get(11)?,
        })
    }
}


impl TryFrom<CompanyMasterSql> for CompanyMaster {
    type Error = DaoError;

    fn try_from(company_master_sql: CompanyMasterSql) -> Result<Self, Self::Error> {
        const DIAGNOSTIC_HELP: &str = "failed conversion of CompanyMasterSql to CompanyMaster";
        let k = CompanyMaster {
            base_master_fields: BaseMasterFields {
                id: company_master_sql.id,
                entity_version_id: company_master_sql.entity_version_id,
                tenant_id: company_master_sql.tenant_id,
                active: company_master_sql.active,
                approval_status: MasterStatusEnum::get_enum_for_value(company_master_sql.approval_status as usize)
                    .context(DIAGNOSTIC_HELP)?,
                remarks: company_master_sql.remarks
                    .map(|remarks| MasterUpdationRemarks::new(&remarks))
                    .transpose()
                    .context(DIAGNOSTIC_HELP)?,
            },
            name: CompanyName::new(&company_master_sql.name)
                .context(DIAGNOSTIC_HELP)?,
            cin: CompanyIdentificationNumber::new(&company_master_sql.cin)
                .context(DIAGNOSTIC_HELP)?,
            audit_metadata: AuditMetadataBase {
                created_by: company_master_sql.created_by,
                updated_by: company_master_sql.updated_by,
                created_at: company_master_sql.created_at,
                updated_at: company_master_sql.updated_at,
            },
        };
        Ok(k)
    }
}