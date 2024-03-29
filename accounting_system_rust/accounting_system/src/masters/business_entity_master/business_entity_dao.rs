use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::{GenericClient, Pool};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::common_utils::db_row_conversion_utils::{convert_row_to_audit_metadata_base, convert_row_to_base_master_fields};
use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_uuid;
use crate::masters::business_entity_master::business_entity_models::{BusinessEntityMaster, BusinessEntityName, BusinessEntityType, CreateBusinessEntityRequest, Email, PhoneNumber};
use crate::masters::company_master::company_master_models::gstin_no::GstinNo;

#[async_trait]
pub trait BusinessEntityDao: Send + Sync {
    async fn create_business_entity(&self, request: &CreateBusinessEntityRequest) -> Result<Uuid, DaoError>;

    async fn get_business_entity(&self, id: &Uuid, tenant_id: &Uuid) -> Result<Option<BusinessEntityMaster>, DaoError>;
    async fn is_business_entity_exist(&self, id: &Uuid, tenant_id: &Uuid) -> Result<bool, DaoError>;
}

const TABLE_NAME: &str = "business_entity";
const SELECT_FIELDS: &str = "id,entity_version_id,tenant_id,active,approval_status,remarks,eligible_supplier,name,email,phone,address_id,gstin,created_by,updated_by,created_at,updated_at";
const QUERY_BY_ID: &str = concatcp!("select ",SELECT_FIELDS," from ",TABLE_NAME," where id=$1 and tenant_id=$2");

struct BusinessEntityDaoImpl {
    postgres_client: Arc<Pool>,
}

impl TryFrom<Row> for BusinessEntityMaster {
    type Error = DaoError;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        let (base_master_fields, next_ind) = convert_row_to_base_master_fields(&row)?;
        let eligible_supplier: bool = row.get(next_ind);
        let e_type: BusinessEntityType = if eligible_supplier {
            BusinessEntityType::EligibleSupplier {
                name: BusinessEntityName::new(row.get(next_ind + 1))?,
                email: Email::new(row.get(next_ind + 2))?,
                phone: PhoneNumber::new(row.get(next_ind + 3))?,
                address_id: row.get(next_ind + 4),
                gstin: GstinNo::new(row.get(next_ind + 5))
                    .context("error during db row conversion")?,
            }
        } else {
            BusinessEntityType::Other {
                name: BusinessEntityName::new(row.get(next_ind + 1))?,
                email: row.get::<usize, Option<&str>>(next_ind + 2)
                    .map(|a| Email::new(a))
                    .transpose()?,
                phone: PhoneNumber::new(row.get(next_ind + 3))?,
                address_id: row.get(next_ind + 4),
                gstin: row.get::<usize, Option<&str>>(next_ind + 5)
                    .map(|a| GstinNo::new(a))
                    .transpose()
                    .context("error during db row conversion")?,
            }
        };
        Ok(BusinessEntityMaster {
            base_master_fields,
            entity_type: e_type,
            audit_metadata: convert_row_to_audit_metadata_base(next_ind + 6, &row)?,
        })
    }
}

#[allow(dead_code)]
pub fn get_business_entity_dao(client: Arc<Pool>) -> Arc<dyn BusinessEntityDao> {
    let a = BusinessEntityDaoImpl {
        postgres_client: client
    };
    Arc::new(a)
}

#[async_trait]
impl BusinessEntityDao for BusinessEntityDaoImpl {
    async fn create_business_entity(&self, r: &CreateBusinessEntityRequest) -> Result<Uuid, DaoError> {
        let k: String = match &r.entity_type {
            BusinessEntityType::EligibleSupplier {
                name, email, phone
                , address_id, gstin
            } => {
                format!("Row('{}','{}',{}::smallint,true,'{}','{}','{}','{address_id}','{}',\
                 '{}')"
                        , r.idempotence_key, r.tenant_id, 1, name.inner(), email.inner()
                        , phone.inner(), gstin.get_str(), r.created_by)
            }
            BusinessEntityType::Other {
                name, email,
                phone, address_id, gstin
            } => {
                format!("Row('{}','{}',{}::smallint,true,'{}',{},'{}',{},{},\
                 '{}')"
                        , r.idempotence_key, r.tenant_id, 1, name.inner(),
                        email.as_ref().map(|a| format!("'{}'", a.inner()))
                            .unwrap_or("null".to_string())
                        , phone.inner(), address_id
                            .map(|a| format!("'{}'", a))
                            .unwrap_or("null".to_string()),
                        gstin.as_ref()
                            .map(|a| format!("'{}'", a.get_str()))
                            .unwrap_or("null".to_string()), r.created_by)
            }
        };
        let simple_query = format!(
            r#"
            begin transaction;
            select create_business_entity({});
            commit;
          "#,
            k
        );
        let conn = self.postgres_client.get().await?;
        let rows = conn.simple_query(simple_query.as_str()).await?;
        parse_db_output_of_insert_create_and_return_uuid(&rows)
    }

    async fn get_business_entity(&self, id: &Uuid, tenant_id: &Uuid) -> Result<Option<BusinessEntityMaster>, DaoError> {
        let query = QUERY_BY_ID;
        let en: Option<BusinessEntityMaster> = self.postgres_client.get().await?
            .query_opt(query, &[&id, &tenant_id]).await?
            .map(|a| a.try_into())
            .transpose()?;
        Ok(en)
    }

    async fn is_business_entity_exist(&self, id: &Uuid, tenant_id: &Uuid) -> Result<bool, DaoError> {
        let row = self.postgres_client.get().await?
            .query_one("select exists (select 1 from business_entity where id=$1 and tenant_id=$2)", &[id, tenant_id])
            .await?;
        let exists: bool = row.get(0);
        Ok(exists)
    }
}


#[cfg(test)]
mod tests {
    use spectral::assert_that;
    use spectral::option::OptionAssertions;
    use uuid::Uuid;

    use crate::accounting::postgres_factory::test_utils_postgres::{get_dao_generic, get_postgres_conn_pool, get_postgres_image_port};
    use crate::masters::business_entity_master::business_entity_dao::{BusinessEntityDao, BusinessEntityDaoImpl};
    use crate::masters::business_entity_master::business_entity_models::tests::{a_create_business_entity_request, SEED_BUSINESS_ENTITY_ID2};
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    #[tokio::test]
    async fn test_is_business_entity_exist() {
        let dao = get_dao_generic(|a| BusinessEntityDaoImpl { postgres_client: a.clone() },None).await;
        let exist = dao.is_business_entity_exist(&SEED_BUSINESS_ENTITY_ID2, &SEED_TENANT_ID).await.unwrap();
        let not_exist = dao.is_business_entity_exist(&Uuid::now_v7(), &SEED_TENANT_ID).await.unwrap();
        assert!(exist);
        assert!(!not_exist);
    }

    #[tokio::test]
    async fn test_create_and_get_dao() {
        let dao = get_dao_generic(|a| BusinessEntityDaoImpl { postgres_client: a.clone() },None).await;
        let be = a_create_business_entity_request(Default::default());
        let p = dao.create_business_entity(&be).await.unwrap();
        let k = dao.get_business_entity(&p, &be.tenant_id).await.unwrap();
        assert_that!(k).is_some();
    }
}
