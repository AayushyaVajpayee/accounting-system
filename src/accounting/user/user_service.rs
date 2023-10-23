use std::sync::Arc;
use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use uuid::Uuid;

use crate::accounting::postgres_factory::get_postgres_conn_pool;
use crate::accounting::user::user_dao::{get_user_dao, UserDao};
use crate::accounting::user::user_models::{CreateUserRequest, User};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait UserService:Send+Sync {
    async fn get_user_by_id(&self, id: Uuid) -> Option<User>;
    async fn create_user(&self, user: &CreateUserRequest) -> Uuid;
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
    async fn get_user_by_id(&self, id: Uuid) -> Option<User> {
        //todo to be cached locally
        self.user_dao.get_user_by_id(id).await
    }

    async fn create_user(&self, user: &CreateUserRequest) -> Uuid {
        self.user_dao.create_user(user).await
    }
}