use postgres::Client;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::accounting::user::user_models::{CreateUserRequest, User};

pub trait UserDao {
    fn get_user_by_id(&mut self, id: &i32) -> Option<User>;
    fn create_user(&mut self, request: &CreateUserRequest) -> i32;
}

pub struct UserDaoPostgresImpl {
    postgres_client: Client,
}

impl UserDao for UserDaoPostgresImpl {
    fn get_user_by_id(&mut self, id: &i32) -> Option<User> {
        let rows = self.postgres_client.query("\
        select id,tenant_id,first_name,last_name,email_id,mobile_number,created_by,updated_by,created_at,updated_at from app_user where id=$1",
                                              &[&id]).unwrap();
        rows.iter().map(|row|
            User {
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
            }
        ).next()
    }

    fn create_user(&mut self, request: &CreateUserRequest) -> i32 {
        self.postgres_client.query(
            "insert into app_user (tenant_id,first_name,last_name,email_id,mobile_number,created_by,updated_by,created_at,updated_at)\
            values ($1,$2,$3,$4,$5,$6,$7,$8,$9) returning id", &[
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
    use postgres::{Client, NoTls};
    use testcontainers::clients;
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;

    use crate::accounting::tenant::tenant_models::a_create_tenant_request;
    use crate::accounting::tenant::tenant_service::get_tenant_service_for_test;
    use crate::accounting::user::user_dao::{UserDao, UserDaoPostgresImpl};
    use crate::accounting::user::user_models::{a_create_user_request, CreateUserRequestTestBuilder};

    fn create_postgres_client(port: u16) -> Client {
        let con_str =
            format!("host=localhost user=postgres password=postgres dbname=postgres port={port}");
        let client = Client::
        connect(&con_str, NoTls)
            .unwrap();
        client
    }

    fn create_schema(client: &mut Client) {
        let path = format!("schema/postgres/schema.sql");
        let fi = std::fs::read_to_string(path).unwrap();
        // println!("{fi}");
        client.simple_query(&fi).unwrap();
    }

    #[test]
    fn test_users() {
        let test_container_client = clients::Cli::default();
        let image = "postgres";
        let image_tag = "latest";
        let generic_postgres = GenericImage::new(image, image_tag)
            .with_wait_for(WaitFor::message_on_stderr("database system is ready to accept connections"))
            .with_env_var("POSTGRES_DB", "postgres")
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres");
        let node = test_container_client.run(generic_postgres);
        let port = node.get_host_port_ipv4(5432);
        let mut postgres_client = create_postgres_client(port);
        create_schema(&mut postgres_client);
        let t1 = a_create_tenant_request(Default::default());
        let mut tenant_service = get_tenant_service_for_test(postgres_client);
        let tenant_id = tenant_service.create_tenant(&t1);
        let user = a_create_user_request(
            CreateUserRequestTestBuilder {
                tenant_id: Some(tenant_id),
                ..Default::default()
            }
        );
        let mut postgres_client = create_postgres_client(port);
        let mut user_dao = UserDaoPostgresImpl { postgres_client };
        let user_id = user_dao.create_user(&user);
        let user = user_dao.get_user_by_id(&user_id);
        println!("{:?}", user);
    }
}