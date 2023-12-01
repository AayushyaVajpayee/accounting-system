use std::sync::Arc;
use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use thiserror::Error;
use uuid::Uuid;

use crate::accounting::postgres_factory::get_postgres_conn_pool;
use crate::accounting::user::user_dao::{get_user_dao, UserDao};
use crate::accounting::user::user_models::{CreateUserRequest, User};
use crate::common_utils::dao_error::DaoError;

#[derive(Debug, Error, PartialEq)]
pub enum UserServiceError {
    #[error(transparent)]
    Db(#[from]DaoError)
}
#[cfg_attr(test, automock)]
#[async_trait]
pub trait UserService:Send+Sync {
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, UserServiceError>;
    async fn create_user(&self, user: &CreateUserRequest) -> Result<Uuid, UserServiceError>;
}

#[allow(dead_code)]
pub fn get_user_service() -> Arc<dyn UserService> {
    let pclient = get_postgres_conn_pool();
    let user_dao = get_user_dao(pclient);
    let user_service = UserServiceImpl {
        user_dao
    };
    Arc::new(user_service)
}

#[allow(dead_code)]
#[cfg(test)]
pub fn get_user_service_for_test(postgres_client: &'static deadpool_postgres::Pool) -> Arc<dyn UserService> {
    let user_dao = get_user_dao(postgres_client);
    let user_service = UserServiceImpl {
        user_dao
    };
    Arc::new(user_service)
}

struct UserServiceImpl {
    user_dao: Arc<dyn UserDao>,
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, UserServiceError> {
        //todo to be cached locally
        self.user_dao.get_user_by_id(id).await.map_err(|a| a.into())
    }

    async fn create_user(&self, user: &CreateUserRequest) -> Result<Uuid, UserServiceError> {
        self.user_dao.create_user(user).await.map_err(|a| a.into())
    }
}