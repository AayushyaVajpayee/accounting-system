use crate::accounting::user::user_dao::UserDao;
use crate::accounting::user::user_models::{CreateUserRequest, User};

pub trait UserService {
    fn get_user_by_id(&mut self, id: &i32) -> Option<User>;
    fn create_user(&mut self, user: &CreateUserRequest) -> i32;
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