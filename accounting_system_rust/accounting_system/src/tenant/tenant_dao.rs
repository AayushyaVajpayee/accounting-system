use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::Pool;
use std::sync::Arc;
use tokio_postgres::{Row};
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::common_utils::dao_error::DaoError;
use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_uuid;
use crate::tenant::tenant_models::{CreateTenantRequest, Tenant};

const SELECT_FIELDS: &str = "id,display_name,created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "tenant";
const BY_ID_QUERY: &str = concatcp!(
    "select ",
    SELECT_FIELDS,
    " from ",
    TABLE_NAME,
    " where id=$1"
);
#[async_trait]
pub trait TenantDao: Send + Sync {
    async fn get_tenant_by_id(&self, id: Uuid) -> Result<Option<Tenant>, DaoError>;
    async fn create_tenant(&self, tenant: &CreateTenantRequest) -> Result<Uuid, DaoError>;
    // async fn update_tenant(&self, tenant: &CreateTenantRequest) -> i64;
    // async fn delete_tenant(&self, tenant_id: &str) -> i64;
}

pub fn get_tenant_dao(client: Arc<Pool>) -> Arc<dyn TenantDao> {
    let td = TenantDaoImpl {
        postgres_client: client,
    };
    Arc::new(td)
}

struct TenantDaoImpl {
    postgres_client: Arc<Pool>,
}

impl TryFrom<&Row> for Tenant {
    type Error = DaoError;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Tenant {
            id: row.get(0),
            display_name: row.get(1),
            audit_metadata: AuditMetadataBase {
                created_by: row.get(2),
                updated_by: row.get(3),
                created_at: row.get(4),
                updated_at: row.get(5),
            },
        })
    }
}

#[async_trait]
impl TenantDao for TenantDaoImpl {
    async fn get_tenant_by_id(&self, id: Uuid) -> Result<Option<Tenant>, DaoError> {
        let query = BY_ID_QUERY;

        let tenant = self
            .postgres_client.get().await?.query(query, &[&id]).await?
            .iter().map(|row| row.try_into()).next().transpose()?;
        Ok(tenant)
    }

    async fn create_tenant(&self, tenant: &CreateTenantRequest) -> Result<Uuid, DaoError> {
        let simple_query = format!(
            r#"
        begin transaction;
        select create_tenant(Row('{}','{}','{}','{}',{},{}));
        commit;
        "#,
            tenant.idempotence_key,
            tenant.display_name,
            tenant.audit_metadata.created_by,
            tenant.audit_metadata.updated_by,
            tenant.audit_metadata.created_at,
            tenant.audit_metadata.updated_at
        );
        let conn = self.postgres_client.get().await?;
        let rows = conn.simple_query(simple_query.as_str()).await?;
        parse_db_output_of_insert_create_and_return_uuid(&rows)
    }

    // async fn update_tenant(&self, _tenant: &CreateTenantRequest) -> i64 {
    //     todo!()
    // }
    //
    // async fn delete_tenant(&self, _tenant_id: &str) -> i64 {
    //     todo!()
    // }
}

#[cfg(test)]
mod tests {
    use log::kv::Source;
    use spectral::assert_that;
    use spectral::prelude::OptionAssertions;

    use crate::accounting::postgres_factory::test_utils_postgres::{
        get_postgres_conn_pool, get_postgres_image_port,
    };
    use crate::tenant::tenant_dao::{TenantDao, TenantDaoImpl};
    use crate::tenant::tenant_models::CreateTenantRequestBuilder;
    use crate::tenant::tenant_models::tests::a_create_tenant_request;

    #[tokio::test]
    async fn should_be_able_to_create_and_fetch_tenant() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let t1 = a_create_tenant_request(Default::default());
        let tenant_dao = TenantDaoImpl { postgres_client: postgres_client.clone() };
        tenant_dao.create_tenant(&t1).await.unwrap();
        let created_tenant_id = tenant_dao.create_tenant(&t1).await.unwrap();
        let fetched_tenant = tenant_dao
            .get_tenant_by_id(created_tenant_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(created_tenant_id, fetched_tenant.id)
    }
    #[tokio::test]
    async fn should_create_tenant_when_only_1_new_request() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let tenant_request = a_create_tenant_request(Default::default());
        let tenant_dao = TenantDaoImpl { postgres_client: postgres_client.clone() };
        let id = tenant_dao.create_tenant(&tenant_request).await.unwrap();
        let tenant = tenant_dao.get_tenant_by_id(id).await.unwrap();
        assert_that(&tenant).is_some().matches(|a| a.id == id);
    }
    #[tokio::test]
    async fn should_return_existing_tenant_when_idempotency_key_is_same_as_earlier_completed_request(
    ) {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let name = "tsting";
        let mut builder = CreateTenantRequestBuilder::default();
        builder.display_name(name.to_string());
        let tenant_request = a_create_tenant_request(builder);
        let tenant_dao = TenantDaoImpl { postgres_client: postgres_client.clone() };
        let id = tenant_dao.create_tenant(&tenant_request).await.unwrap();
        let id1 = tenant_dao.create_tenant(&tenant_request).await.unwrap();
        assert_that!(&id).is_equal_to(id1);
        let number_of_tenants_created: i64 = postgres_client
            .get()
            .await
            .unwrap()
            .query(
                "select count(id) from tenant where display_name=$1",
                &[&name],
            )
            .await
            .unwrap()
            .iter()
            .map(|a| a.get(0))
            .next()
            .unwrap();
        assert_that!(number_of_tenants_created).is_equal_to(1)
        // let tenant = tenant_dao.get_tenant_by_id(id).await.unwrap();
    }
    async fn should_return_existing_tenant_and_create_no_new_tenant_when_idempotency_key_is_same_as_earlier_completed_request(
    ) {
       //hard to perform test with my current knowledge. Have performed this manually
    }

}
