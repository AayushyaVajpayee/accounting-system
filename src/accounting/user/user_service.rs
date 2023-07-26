use postgres::Client;

use crate::accounting::postgres_factory::create_postgres_client;
use crate::accounting::user::user_dao::{get_user_dao, UserDao};
use crate::accounting::user::user_models::{CreateUserRequest, User};

pub trait UserService {
    fn get_user_by_id(&mut self, id: &i32) -> Option<User>;
    fn create_user(&mut self, user: &CreateUserRequest) -> i32;
}

#[allow(dead_code)]
pub fn get_user_service() -> Box<dyn UserService> {
    let pclient = create_postgres_client();
    let user_dao = get_user_dao(pclient);
    let user_service = UserServiceImpl {
        user_dao: user_dao
    };
    Box::new(user_service)
}

#[allow(dead_code)]
#[cfg(test)]
pub fn get_user_service_for_test(postgres_client: Client) -> Box<dyn UserService> {
    let user_dao = get_user_dao(postgres_client);
    let user_service = UserServiceImpl {
        user_dao: user_dao
    };
    Box::new(user_service)
}

struct UserServiceImpl {
    user_dao: Box<dyn UserDao>,
}

impl UserService for UserServiceImpl {
    fn get_user_by_id(&mut self, id: &i32) -> Option<User> {
        self.user_dao.get_user_by_id(id)
    }

    fn create_user(&mut self, user: &CreateUserRequest) -> i32 {
        self.user_dao.create_user(user)
    }
}