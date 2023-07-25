use std::sync::OnceLock;
use postgres::{Client, Row};

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::accounting::user::user_models::{CreateUserRequest, User};

const SELECT_FIELDS:&str="id,tenant_id,first_name,last_name,email_id,mobile_number,created_by,updated_by,created_at,updated_at";
const TABLE_NAME: &str ="app_user";
static BY_ID_QUERY:OnceLock<String> = OnceLock::new();
static INSERT_STATEMENT:OnceLock<String> = OnceLock::new();

pub trait UserDao {
    fn get_user_by_id(&mut self, id: &i32) -> Option<User>;
    fn create_user(&mut self, request: &CreateUserRequest) -> i32;
}

pub struct UserDaoPostgresImpl {
    postgres_client: Client,
}

impl TryFrom<&Row> for User{
    type Error = ();

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok( User {
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

impl UserDaoPostgresImpl {
    fn get_user_by_id_query()->&'static String{
        BY_ID_QUERY.get_or_init(||{
            format!("select {} from {} where id=$1",SELECT_FIELDS,TABLE_NAME)
        })
    }
    fn create_insert_statement()->&'static String{
        INSERT_STATEMENT.get_or_init(||{
            format!("insert into {} ({}) values \
            (DEFAULT,$1,$2,$3,$4,$5,$6,$7,$8,$9) returning id",
            TABLE_NAME,SELECT_FIELDS)
        })
    }
}

#[allow(dead_code)]
pub fn get_user_dao(client: Client) -> Box<dyn UserDao> {
    let user_dao = UserDaoPostgresImpl {
        postgres_client: client
    };
    Box::new(user_dao)
}

impl UserDao for UserDaoPostgresImpl {
    fn get_user_by_id(&mut self, id: &i32) -> Option<User> {
        let query = UserDaoPostgresImpl::get_user_by_id_query();
        let rows = self.postgres_client.query(query,
                                              &[&id]).unwrap();
        rows.iter().map(|row|
            row.try_into().unwrap()
        ).next()
    }

    fn create_user(&mut self, request: &CreateUserRequest) -> i32 {
        let query = UserDaoPostgresImpl::create_insert_statement();
        self.postgres_client.query(
            query, &[
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
        ).unwrap()
            .iter()
            .map(|row| row.get(0)).next().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::accounting::user::user_dao::{UserDao, UserDaoPostgresImpl};
    use crate::accounting::user::user_models::{a_create_user_request, CreateUserRequestTestBuilder};
    use crate::test_utils::test_utils_postgres::{create_postgres_client, get_postgres_image_port};

    #[test]
    fn should_be_able_to_create_and_fetch_users() {
        let port = get_postgres_image_port();
        let user = a_create_user_request(
            CreateUserRequestTestBuilder {
                tenant_id: Some(1),
                ..Default::default()
            }
        );
        let postgres_client = create_postgres_client(port);
        let mut user_dao = UserDaoPostgresImpl { postgres_client };
        let user_id = user_dao.create_user(&user);
        let user = user_dao.get_user_by_id(&user_id).unwrap();
        assert_eq!(user.id, user_id);
    }
}