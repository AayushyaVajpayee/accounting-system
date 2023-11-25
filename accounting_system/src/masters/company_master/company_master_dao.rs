use std::sync::Arc;
use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::{GenericClient, Pool, PoolError};
use tokio_postgres::error::{DbError, SqlState};
use tokio_postgres::Row;
use tracing::{error, instrument};
use uuid::Uuid;
#[cfg(test)]
use mockall::automock;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::common_utils::dao_error::DaoError;
use crate::masters::company_master::company_master_dao::DaoError::InvalidEntityToDbRowConversion;
use crate::masters::company_master::company_master_model::MasterStatusEnum::{Approved, Deleted};
use crate::masters::company_master::company_master_model::{
    BaseMasterFields, CompanyIdentificationNumber, CompanyMaster, CompanyName, MasterStatusEnum,
    MasterUpdationRemarks,
};

const SELECT_FIELDS:&str ="id,entity_version_id,tenant_id,active,approval_status,remarks,name,cin,created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "company_master";

const GET_BY_ID: &str = concatcp!(
    "select ",
    SELECT_FIELDS,
    " from ",
    TABLE_NAME,
    " where id=$1 and tenant_id=$2 and approval_status=",
    Approved as i32
);
const GET_ALL_FOR_TENANT: &str = concatcp!(
    "select ",
    SELECT_FIELDS,
    " from ",
    TABLE_NAME,
    " where tenant_id=$1 and approval_status=",
    Approved as i32
);

const INSERT: &str = concatcp!(
    "insert into ",
    TABLE_NAME,
    "(",
    SELECT_FIELDS,
    ")",
    " values ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)"
);

const SOFT_DELETE: &str = concatcp!(
    "update ",
    TABLE_NAME,
    " set approval_status=",
    Deleted as i32,
    " ,entity_version_id=entity_version_id+1,remarks=$4,updated_by=$5,updated_at=extract(epoch from now()) * 1000000",
    " where id=$1 and tenant_id=$2 and entity_version_id=$3"
);
pub struct CompanyMasterDaoPostgresImpl {
    postgres_client: &'static Pool,
}

pub fn get_company_master_dao(pool: &'static Pool) -> Arc<dyn CompanyMasterDao> {
    let dao = CompanyMasterDaoPostgresImpl {
        postgres_client: pool,
    };
    Arc::new(dao)
}
impl TryFrom<&Row> for CompanyMaster {
    type Error = DaoError;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let remarks: Option<&str> = row.get(5);
        let remarks = if remarks.is_some() {
            let k = MasterUpdationRemarks::new(remarks.unwrap())
                .map_err(|a| InvalidEntityToDbRowConversion(a))?;
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
                )
                .map_err(InvalidEntityToDbRowConversion)?,
                remarks,
            },
            name: CompanyName::new(row.get(6)).map_err(InvalidEntityToDbRowConversion)?,
            cin: CompanyIdentificationNumber::new(row.get(7))
                .map_err(InvalidEntityToDbRowConversion)?,
            audit_metadata: AuditMetadataBase {
                created_by: row.get(8),
                updated_by: row.get(9),
                created_at: row.get(10),
                updated_at: row.get(11),
            },
        })
    }
}


#[cfg_attr(test, automock)]
#[async_trait]
pub trait CompanyMasterDao:Send+Sync {
    async fn get_company_by_id(
        &self,
        tenant_id: Uuid,
        company_id: Uuid,
    ) -> Result<Option<CompanyMaster>, DaoError>;
    async fn get_all_companies_for_tenant(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<CompanyMaster>, DaoError>;
    async fn create_new_company_for_tenant(&self, entity: &CompanyMaster) -> Result<u64, DaoError>;
    // async fn update_company_data_for_tenant(&self);
    async fn soft_delete_company_for_tenant(
        &self,
        tenant_id: Uuid,
        company_id: Uuid,
        entity_version_id: i32,
        remarks: &MasterUpdationRemarks,
        updated_by: Uuid,
    ) -> Result<u64, DaoError>;
}

#[async_trait]
impl CompanyMasterDao for CompanyMasterDaoPostgresImpl {
    async fn get_company_by_id(
        &self,
        tenant_id: Uuid,
        company_id: Uuid,
    ) -> Result<Option<CompanyMaster>, DaoError> {
        let conn = self.postgres_client.get().await?;
        let p: Option<CompanyMaster> = conn
            .query(GET_BY_ID, &[&company_id, &tenant_id])
            .await?
            .iter()
            .map(|a| a.try_into())
            .next()
            .transpose()?;
        Ok(p)
    }

    async fn get_all_companies_for_tenant(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<CompanyMaster>, DaoError> {
        let conn = self.postgres_client.get().await?;
        let rows = conn
            .query(GET_ALL_FOR_TENANT, &[&tenant_id])
            .await?
            .iter()
            .map(|a| a.try_into())
            .collect::<Result<Vec<CompanyMaster>, DaoError>>()?;
        Ok(rows)
    }
    #[instrument(skip(self))]
    async fn create_new_company_for_tenant(&self, entity: &CompanyMaster) -> Result<u64, DaoError> {
        let conn = self.postgres_client.get().await?;
        let inserted_rows = conn
            .execute(
                INSERT,
                &[
                    &entity.base_master_fields.id,
                    &entity.base_master_fields.entity_version_id,
                    &entity.base_master_fields.tenant_id,
                    &entity.base_master_fields.active,
                    &(entity.base_master_fields.approval_status as i16),
                    &entity
                        .base_master_fields
                        .remarks
                        .as_ref()
                        .map(|a| a.get_str()),
                    &entity.name.get_str(),
                    &entity.cin.get_str(),
                    &entity.audit_metadata.created_by,
                    &entity.audit_metadata.updated_by,
                    &entity.audit_metadata.created_at,
                    &entity.audit_metadata.updated_at,
                ],
            )
            .await?;
        Ok(inserted_rows)
    }

    async fn soft_delete_company_for_tenant(
        &self,
        tenant_id: Uuid,
        company_id: Uuid,
        entity_version_id: i32,
        remarks: &MasterUpdationRemarks,
        updated_by: Uuid,
    ) -> Result<u64, DaoError> {
        let conn = self.postgres_client.get().await?;

        let updated_rows = conn
            .execute(
                SOFT_DELETE,
                &[
                    &company_id,
                    &tenant_id,
                    &entity_version_id,
                    &remarks.get_str(),
                    &updated_by,
                ],
            )
            .await?;
        Ok(updated_rows)
    }
}

#[cfg(test)]
mod tests {
    use spectral::assert_that;
    use spectral::option::OptionAssertions;
    use spectral::prelude::VecAssertions;

    use crate::accounting::postgres_factory::test_utils_postgres::{
        get_postgres_conn_pool, get_postgres_image_port,
    };
    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::audit_table::audit_service::get_audit_service_for_tests;
    use crate::masters::company_master::company_master_dao::{
        CompanyMasterDao, CompanyMasterDaoPostgresImpl, TABLE_NAME,
    };
    use crate::masters::company_master::company_master_model::test_data::{
        a_base_master_field, a_company_master, BaseMasterFieldsTestDataBuilder,
        CompanyMasterTestDataBuilder,
    };
    use crate::masters::company_master::company_master_model::MasterStatusEnum::Approved;
    use crate::masters::company_master::company_master_model::MasterUpdationRemarks;

    #[tokio::test]
    async fn test_insert_and_get_for_company_master() {
        //todo figure out how to implement idempotency in inserts and how to test them
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let dao = CompanyMasterDaoPostgresImpl { postgres_client };
        let company_master = a_company_master(CompanyMasterTestDataBuilder {
            base_master_fields: Some(a_base_master_field(BaseMasterFieldsTestDataBuilder {
                approval_status: Some(Approved),
                ..Default::default()
            })),
            ..Default::default()
        });
        dao.create_new_company_for_tenant(&company_master)
            .await
            .unwrap();
        let k = dao
            .get_company_by_id(
                company_master.base_master_fields.tenant_id,
                company_master.base_master_fields.id,
            )
            .await
            .unwrap();
        println!("{:?}", k);
        assert_that!(k)
            .is_some()
            .map(|a| &a.base_master_fields.id)
            .is_equal_to(company_master.base_master_fields.id);
    }

    #[tokio::test]
    async fn test_soft_delete_for_company_master() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let dao = CompanyMasterDaoPostgresImpl { postgres_client };
        let company_master = a_company_master(CompanyMasterTestDataBuilder {
            base_master_fields: Some(a_base_master_field(BaseMasterFieldsTestDataBuilder {
                approval_status: Some(Approved),
                ..Default::default()
            })),
            ..Default::default()
        });
        dao.create_new_company_for_tenant(&company_master)
            .await
            .unwrap();
        let p = dao
            .get_company_by_id(
                company_master.base_master_fields.tenant_id,
                company_master.base_master_fields.id,
            )
            .await
            .unwrap();
        let updated_rows = dao
            .soft_delete_company_for_tenant(
                company_master.base_master_fields.tenant_id,
                company_master.base_master_fields.id,
                p.unwrap().base_master_fields.entity_version_id,
                &MasterUpdationRemarks::new("unit testing delete function").unwrap(),
                *SEED_USER_ID,
            )
            .await
            .unwrap();
        assert_that!(updated_rows).is_equal_to(1);
        let k = get_audit_service_for_tests(postgres_client);
        let ppp = k
            .get_audit_logs_for_id_and_table(company_master.base_master_fields.id, TABLE_NAME)
            .await;
        assert_that!(ppp).has_length(1);
    }

    #[tokio::test]
    async fn test_get_all_companies_for_tenant() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let dao = CompanyMasterDaoPostgresImpl { postgres_client };
        let company_master = a_company_master(CompanyMasterTestDataBuilder {
            base_master_fields: Some(a_base_master_field(BaseMasterFieldsTestDataBuilder {
                approval_status: Some(Approved),
                ..Default::default()
            })),
            ..Default::default()
        });
        dao.create_new_company_for_tenant(&company_master)
            .await
            .unwrap();
        let p = dao
            .get_all_companies_for_tenant(company_master.base_master_fields.tenant_id)
            .await
            .unwrap();
        assert!(!p.is_empty())
    }
}
