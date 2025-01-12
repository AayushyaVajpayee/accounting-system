#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use speculoos::assert_that;
    use speculoos::option::OptionAssertions;
    use tokio_postgres::SimpleQueryMessage;
    use uuid::Uuid;

    use crate::accounting::postgres_factory::test_utils_postgres::{
        get_dao_generic, get_postgres_conn_pool, get_postgres_image_port,
    };
    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::invoicing::payment_term::payment_term_models::tests::SEED_PAYMENT_TERM_ID;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    #[tokio::test]
    async fn should_retrieve_existing_get_or_create_payment_term() {
        let postgres_client = get_dao_generic(|a| a, None).await;
        let query_form = format!(
            r#"
            begin transaction;
            select get_or_create_payment_term({},{},{},'{}','{}');
            commit;
        "#,
            5, 0, 0, *SEED_TENANT_ID, *SEED_USER_ID
        );
        let p = postgres_client
            .get()
            .await
            .unwrap()
            .simple_query(&query_form)
            .await
            .unwrap();
        let ak = p.get(2).unwrap();
        match ak {
            SimpleQueryMessage::Row(a) => {
                let p: Option<&str> = a.get(0);
                assert_that!(p)
                    .is_some()
                    .matches(|a| **a == *SEED_PAYMENT_TERM_ID.to_string().as_str());
            }
            SimpleQueryMessage::CommandComplete(_) => {
                unreachable!();
            }
            _ => {
                unreachable!();
            }
        }
    }
    #[tokio::test]
    async fn should_create_new_get_or_create_payment_term() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let query_form = format!(
            r#"
            begin transaction;
            select get_or_create_payment_term({},{},{},'{}','{}');
            commit;
        "#,
            5, 02, 0, *SEED_TENANT_ID, *SEED_USER_ID
        );
        let p = postgres_client
            .get()
            .await
            .unwrap()
            .simple_query(&query_form)
            .await
            .unwrap();
        let ak = p.get(2).unwrap();
        match ak {
            SimpleQueryMessage::Row(a) => {
                let p: Option<&str> = a.get(0);
                assert_that!(p)
                    .is_some()
                    .matches(|a| **a != *SEED_PAYMENT_TERM_ID.to_string().as_str());
                let id = Uuid::from_str(p.unwrap()).unwrap();
                let q = postgres_client
                    .get()
                    .await
                    .unwrap()
                    .query_opt("select * from payment_term where id=$1", &[&id])
                    .await
                    .unwrap();
                assert_that!(q).is_some();
            }
            SimpleQueryMessage::CommandComplete(_) => {
                unreachable!();
            }
            _ => {
                unreachable!();
            }
        }
    }
}
