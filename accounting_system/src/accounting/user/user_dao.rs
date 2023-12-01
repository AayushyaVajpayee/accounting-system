use std::sync::Arc;

use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::Pool;
use tokio_postgres::Row;
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::accounting::user::user_models::{CreateUserRequest, User};
use crate::common_utils::dao_error::DaoError;

const SELECT_FIELDS: &str = "id,tenant_id,first_name,last_name,email_id,mobile_number,created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "app_user";
const BY_ID_QUERY: &str = concatcp!("select ",SELECT_FIELDS," from ",TABLE_NAME," where id=$1");
const INSERT_STATEMENT: &str = concatcp!("insert into ",TABLE_NAME," (",SELECT_FIELDS,")"," values  ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) returning id");


#[async_trait]
pub trait UserDao: Send + Sync {
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, DaoError>;
    async fn create_user(&self, request: &CreateUserRequest) -> Result<Uuid, DaoError>;
}

pub struct UserDaoPostgresImpl {
    postgres_client: &'static Pool,
}

impl TryFrom<&Row> for User {
    type Error = DaoError;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(User {
            id: row.get(0),
            tenant_id: row.get(1),
            first_name: row.get(2),
            last_name: row.get(3),
            email_id: row.get(4),
            mobile_number: row.get(5),
            audit_metadata: AuditMetadataBase {
                created_by: row.get(6),
                updated_by: row.get(7),
                created_at: row.get(8),
                updated_at: row.get(9),
            },
        })
    }
}


#[allow(dead_code)]
pub fn get_user_dao(client: &'static Pool) -> Arc<dyn UserDao> {
    let user_dao = UserDaoPostgresImpl {
        postgres_client: client
    };
    Arc::new(user_dao)
}

#[async_trait]
impl UserDao for UserDaoPostgresImpl {
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, DaoError> {
        let rows = self.postgres_client.get()
            .await?
            .query(BY_ID_QUERY,
                   &[&id]).await?;
        rows.iter().map(|row|
            row.try_into()
        ).next().transpose()
    }

    async fn create_user(&self, request: &CreateUserRequest) -> Result<Uuid, DaoError> {
        let id = Uuid::now_v7();
        let k = self.postgres_client.get().await?.query(
            INSERT_STATEMENT, &[
                &id,
                &request.tenant_id,
                &request.first_name,
                &request.last_name,
                &request.email_id,
                &request.mobile_number,
                &request.audit_metadata.created_by,
                &request.audit_metadata.updated_by,
                &request.audit_metadata.created_at,
                &request.audit_metadata.updated_at
            ],
        ).await?
            .iter()
            .map(|row| row.get(0)).next().unwrap();
        Ok(k)
    }
}

#[cfg(test)]
mod tests {
    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::accounting::user::user_dao::{UserDao, UserDaoPostgresImpl};
    use crate::accounting::user::user_models::tests::{a_create_user_request, CreateUserRequestTestBuilder};
    use crate::tenant::tenant_models::SEED_TENANT_ID;

    #[tokio::test]
    async fn should_be_able_to_create_and_fetch_users() {
        let port = get_postgres_image_port().await;
        let user = a_create_user_request(
            CreateUserRequestTestBuilder {
                tenant_id: Some(*SEED_TENANT_ID),
                ..Default::default()
            }
        );
        let postgres_client = get_postgres_conn_pool(port).await;
        let user_dao = UserDaoPostgresImpl { postgres_client };
        let user_id = user_dao.create_user(&user).await.unwrap();
        let user = user_dao.get_user_by_id(user_id).await.unwrap().unwrap();
        assert_eq!(user.id, user_id);
    }
}