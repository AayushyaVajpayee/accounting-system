use async_trait::async_trait;
use uuid::Uuid;

use crate::accounting::postgres_factory::get_postgres_conn_pool;
use crate::accounting::user::user_dao::{get_user_dao, UserDao};
use crate::accounting::user::user_models::{CreateUserRequest, User};

#[async_trait]
pub trait UserService {
    async fn get_user_by_id(&self, id: Uuid) -> Option<User>;
    async fn create_user(&self, user: &CreateUserRequest) -> Uuid;
}

#[allow(dead_code)]
pub fn get_user_service() -> Box<dyn UserService + Send + Sync> {
    let pclient = get_postgres_conn_pool();
    let user_dao = get_user_dao(pclient);
    let user_service = UserServiceImpl {
        user_dao
    };
    Box::new(user_service)
}

#[allow(dead_code)]
#[cfg(test)]
pub fn get_user_service_for_test(postgres_client: &'static deadpool_postgres::Pool) -> Box<dyn UserService + Send + Sync> {
    let user_dao = get_user_dao(postgres_client);
    let user_service = UserServiceImpl {
        user_dao
    };
    Box::new(user_service)
}

struct UserServiceImpl {
    user_dao: Box<dyn UserDao + Send + Sync>,
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn get_user_by_id(&self, id: Uuid) -> Option<User> {
        self.user_dao.get_user_by_id(id).await
    }

    async fn create_user(&self, user: &CreateUserRequest) -> Uuid {
        self.user_dao.create_user(user).await
    }
}