use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use deadpool_postgres::Pool;
#[cfg(test)]
use mockall::automock;
use moka::future::Cache;
use thiserror::Error;
use uuid::Uuid;

use crate::accounting::user::user_dao::{get_user_dao, UserDao};
use crate::accounting::user::user_models::{CreateUserRequest, User};
use crate::common_utils::cache_utils::get_or_fetch_entity;
use crate::common_utils::dao_error::DaoError;
use crate::tenant::tenant_service::SUPER_USER_ID;

#[derive(Debug, Error)]
pub enum UserServiceError {
    #[error("validation failures \n {}", .0.join("\n"))]
    Validation(Vec<String>),
    #[error(transparent)]
    Db(#[from] DaoError),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_user_by_id(
        &self,
        id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Option<Arc<User>>, UserServiceError>;
    async fn create_user(
        &self,
        user: &CreateUserRequest,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Uuid, UserServiceError>;
}

#[allow(dead_code)]
pub fn get_user_service(arc: Arc<Pool>) -> Arc<dyn UserService> {
    let user_dao = get_user_dao(arc);
    let cache: Cache<(Uuid, Uuid), Arc<User>> = Cache::builder()
        .max_capacity(2000)
        .time_to_live(Duration::from_secs(300))
        .build();
    let user_service = UserServiceImpl {
        user_dao,
        cache_by_tenant_id_and_id: cache,
    };
    Arc::new(user_service)
}

struct UserServiceImpl {
    user_dao: Arc<dyn UserDao>,
    cache_by_tenant_id_and_id: Cache<(Uuid, Uuid), Arc<User>>,
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn get_user_by_id(
        &self,
        id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Option<Arc<User>>, UserServiceError> {
        let fetch = async {
            let p = self.user_dao.get_user_by_id(id, tenant_id).await?;
            Ok(p)
        };
        get_or_fetch_entity(tenant_id, id, &self.cache_by_tenant_id_and_id, fetch).await
    }

    async fn create_user(
        &self,
        user: &CreateUserRequest,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Uuid, UserServiceError> {
        let mut validations: Vec<String> = Vec::new();
        if user_id != *SUPER_USER_ID && tenant_id != user.tenant_id {
            validations.push(format!("This user and tenant does not have authorisation to create user for other tenant id {} .\
             Only allowed tenant id {}"
                                     , user.tenant_id, tenant_id));
        }
        if !validations.is_empty() {
            return Err(UserServiceError::Validation(validations));
        }
        self.user_dao
            .create_user(user, user_id)
            .await
            .map_err(|a| a.into())
    }
}
