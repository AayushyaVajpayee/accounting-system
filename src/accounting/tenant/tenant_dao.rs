use std::sync::OnceLock;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use tokio_postgres::Row;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::accounting::tenant::tenant_models::{CreateTenantRequest, Tenant};

const SELECT_FIELDS: &str = "id,display_name,created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "tenant";
static BY_ID_QUERY: OnceLock<String> = OnceLock::new();
static INSERT_STATEMENT: OnceLock<String> = OnceLock::new();

#[async_trait]
pub trait TenantDao {
    async fn get_tenant_by_id(&self, id: &i32) -> Option<Tenant>;
    async fn create_tenant(&self, tenant: &CreateTenantRequest) -> i32;
    async fn update_tenant(&self, tenant: &CreateTenantRequest) -> i64;
    async fn delete_tenant(&self, tenant_id: &str) -> i64;
}

pub fn get_tenant_dao(client: &'static Pool) -> Box<dyn TenantDao + Send + Sync> {
    let td = TenantDaoImpl {
        postgres_client: client
    };
    Box::new(td)
}

struct TenantDaoImpl {
    postgres_client: &'static Pool,
}

impl TryFrom<&Row> for Tenant {
    type Error = ();

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

impl TenantDaoImpl {
    fn get_tenant_by_id_query() -> &'static String {
        BY_ID_QUERY.get_or_init(|| {
            format!("select {} from {} where id=$1", SELECT_FIELDS, TABLE_NAME)
        })
    }

    fn create_insert_statement() -> &'static String {
        INSERT_STATEMENT.get_or_init(|| {
            format!("insert into {} ({}) values\
             (DEFAULT,$1,$2,$3,$4,$5) returning id", TABLE_NAME, SELECT_FIELDS)
        })
    }
}

#[async_trait]
impl TenantDao for TenantDaoImpl {
    async fn get_tenant_by_id(&self, id: &i32) -> Option<Tenant> {
        let query = TenantDaoImpl::get_tenant_by_id_query();

        let k = self.postgres_client.get().await.unwrap()
            .query(query
                   , &[id])
            .await.unwrap();

        k.iter().map(|row|
            row.try_into().unwrap()
        ).next()
    }

    async fn create_tenant(&self, tenant: &CreateTenantRequest) -> i32 {
        let query = TenantDaoImpl::create_insert_statement();
        self.postgres_client.get().await.unwrap().query(
            query, &[
                &tenant.display_name,
                &tenant.audit_metadata.created_by,
                &tenant.audit_metadata.updated_by,
                &tenant.audit_metadata.created_at,
                &tenant.audit_metadata.updated_at
            ],
        ).await.unwrap().iter().map(|row| row.get(0)).next().unwrap()
    }

    async fn update_tenant(&self, _tenant: &CreateTenantRequest) -> i64 {
        todo!()
    }

    async fn delete_tenant(&self, _tenant_id: &str) -> i64 {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use crate::accounting::tenant::tenant_dao::{TenantDao, TenantDaoImpl};
    use crate::accounting::tenant::tenant_models::a_create_tenant_request;
    use crate::test_utils::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};

    #[tokio::test]
    async fn should_be_able_to_create_and_fetch_tenant() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let t1 = a_create_tenant_request(Default::default());
        let mut tenant_dao = TenantDaoImpl { postgres_client };
        tenant_dao.create_tenant(&t1).await;
        let created_tenant_id = tenant_dao.create_tenant(&t1).await;
        let fetched_tenant = tenant_dao.get_tenant_by_id(&created_tenant_id).await.unwrap();
        assert_eq!(created_tenant_id, fetched_tenant.id)
    }
}