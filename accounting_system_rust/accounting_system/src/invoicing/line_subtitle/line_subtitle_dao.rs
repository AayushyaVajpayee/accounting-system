#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use speculoos::assert_that;
    use speculoos::option::OptionAssertions;
    use tokio_postgres::SimpleQueryMessage;
    use uuid::Uuid;
    use xxhash_rust::xxh32;

    use crate::accounting::postgres_factory::test_utils_postgres::{
        get_dao_generic, get_postgres_conn_pool, get_postgres_image_port,
    };
    use crate::invoicing::line_subtitle::line_subtitle_models::tests::SEED_SUBTITLE_ID;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    #[tokio::test]
    async fn should_retrieve_existing_line_subtitle() {
        let postgres_client = get_dao_generic(|a| a, None).await;
        let mut hasher = xxh32::Xxh32::new(0);
        hasher.update("line subtitle".as_bytes());
        let xxhash = hasher.digest();
        let query_form = format!(
            r#"
            begin transaction;
            select get_or_create_line_subtitle('{}','{}',{});
            commit;
        "#,
            "line subtitle", *SEED_TENANT_ID, xxhash
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
                    .matches(|a| **a == *SEED_SUBTITLE_ID.to_string().as_str());
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
    async fn should_create_new_line_subtitle() {
        let postgres_client = get_dao_generic(|a| a, None).await;
        let mut hasher = xxh32::Xxh32::new(0);
        hasher.update("some subtitle".as_bytes());
        let xxhash = hasher.digest();
        let query_form = format!(
            r#"
            begin transaction;
            select get_or_create_line_subtitle('{}','{}',{});
            commit;
        "#,
            "some subtitle", *SEED_TENANT_ID, xxhash
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
                    .matches(|a| **a != *SEED_SUBTITLE_ID.to_string().as_str());
                let id = Uuid::from_str(p.unwrap()).unwrap();
                let q = postgres_client
                    .get()
                    .await
                    .unwrap()
                    .query_opt("select * from line_subtitle where id=$1", &[&id])
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
