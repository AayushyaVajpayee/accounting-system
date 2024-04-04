use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::Pool;
use std::sync::Arc;
use anyhow::Context;
use tokio_postgres::Row;
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::accounting::user::user_models::{CreateUserRequest, User};
use crate::common_utils::dao_error::DaoError;
use crate::common_utils::utils::{get_current_indian_standard_time, get_current_time_us, parse_db_output_of_insert_create_and_return_uuid};

const SELECT_FIELDS: &str = "id,tenant_id,first_name,last_name,email_id,mobile_number,created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str = "app_user";
const BY_ID_QUERY: &str = concatcp!("select ",SELECT_FIELDS," from ",TABLE_NAME," where id=$1 and tenant_id=$2");


#[async_trait]
pub trait UserDao: Send + Sync {
    async fn get_user_by_id(&self, id: Uuid,tenant_id: Uuid) -> Result<Option<User>, DaoError>;
    async fn create_user(&self, request: &CreateUserRequest,user_id:Uuid) -> Result<Uuid, DaoError>;
}

pub struct UserDaoPostgresImpl {
    postgres_client: Arc<Pool>,
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
pub fn get_user_dao(client: Arc<Pool>) -> Arc<dyn UserDao> {
    let user_dao = UserDaoPostgresImpl {
        postgres_client: client
    };
    Arc::new(user_dao)
}

#[async_trait]
impl UserDao for UserDaoPostgresImpl {
    async fn get_user_by_id(&self, id: Uuid, tenant_id:Uuid) -> Result<Option<User>, DaoError> {
        let rows = self.postgres_client.get()
            .await?
            .query(BY_ID_QUERY,
                   &[&id,&tenant_id]).await?;
        rows.iter().map(|row|
            row.try_into()
        ).next().transpose()
    }

    async fn create_user(&self, request: &CreateUserRequest,user_id:Uuid) -> Result<Uuid, DaoError> {
        let simple_query = format!(r#"
        begin transaction;
        select create_app_user(Row('{}','{}','{}',{},{},{},'{}','{}',{},{}));
        commit;
        "#,
                                   request.idempotence_key,
                                   request.tenant_id,
                                   request.first_name,
                                   request.last_name.as_ref()
                                       .map(|a| format!("'{}'", a))
                                       .unwrap_or_else(|| "null".to_string()),
                                   request.email_id.as_ref()
                                       .map(|a| format!("'{}'", a))
                                       .unwrap_or_else(|| "null".to_string()),
                                   request.mobile_number.as_ref()
                                       .map(|a| format!("'{}'", a))
                                       .unwrap_or_else(|| "null".to_string()),
                                   user_id,
                                   user_id,
                                   get_current_time_us().context("error fetching system time")?,
                                   get_current_time_us().context("error fetching system time")?
        );
        let k = self.postgres_client.get().await?
            .simple_query(simple_query.as_str())
            .await?;
        parse_db_output_of_insert_create_and_return_uuid(&k)
    }
}

#[cfg(test)]
mod tests {
    use spectral::assert_that;
    use spectral::option::OptionAssertions;
    use crate::accounting::user::user_models::SEED_USER_ID;

    use crate::accounting::postgres_factory::test_utils_postgres::{get_dao_generic, get_postgres_image_port};
    use crate::accounting::user::user_dao::{UserDao, UserDaoPostgresImpl};
    use crate::accounting::user::user_models::tests::{a_create_user_request, CreateUserRequestTestBuilder};
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    #[tokio::test]
    async fn should_be_able_to_create_and_fetch_users() {
        let user_dao = get_dao_generic(|a| UserDaoPostgresImpl { postgres_client: a.clone() },None)
            .await;
        let user = a_create_user_request(
            CreateUserRequestTestBuilder {
                tenant_id: Some(*SEED_TENANT_ID),
                ..Default::default()
            }
        );
        let user_id = user_dao.create_user(&user,*SEED_USER_ID).await.unwrap();
        let user = user_dao.get_user_by_id(user_id,*SEED_TENANT_ID).await.unwrap().unwrap();
        assert_eq!(user.id, user_id);
    }


    #[tokio::test]
    async fn should_create_account_when_only_1_new_request() {
        let user_dao = get_dao_generic(|a| UserDaoPostgresImpl { postgres_client: a.clone() },None)
            .await;
        let user_request = a_create_user_request(Default::default());
        let id = user_dao.create_user(&user_request,*SEED_USER_ID).await.unwrap();
        let acc = user_dao.get_user_by_id(id,*SEED_TENANT_ID).await.unwrap();
        assert_that!(acc).is_some();
    }

    #[tokio::test]
    async fn should_return_existing_account_when_idempotency_key_is_same_as_earlier_completed_request() {
        let user_dao = get_dao_generic(|a| UserDaoPostgresImpl { postgres_client: a.clone() },None)
            .await;
        let name = "tsting";
        let user_request =
            a_create_user_request(
                CreateUserRequestTestBuilder {
                    first_name: Some(name.to_string()),
                    ..Default::default()
                });
        let id = user_dao.create_user(&user_request,*SEED_USER_ID).await.unwrap();
        let id2 = user_dao.create_user(&user_request,*SEED_USER_ID).await.unwrap();
        assert_that!(&id).is_equal_to(id2);
        let number_of_users_created: i64 = user_dao.postgres_client
            .get()
            .await
            .unwrap()
            .query(
                "select count(id) from app_user where first_name=$1",
                &[&name],
            )
            .await
            .unwrap()
            .iter()
            .map(|a| a.get(0))
            .next()
            .unwrap();
        assert_that!(number_of_users_created).is_equal_to(1)
        ;
    }
}