




#[cfg(test)]
mod tests{
    use spectral::assert_that;
    use tokio_postgres::SimpleQueryMessage;
    use xxhash_rust::xxh32;
    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::tenant::tenant_models::SEED_TENANT_ID;

    #[tokio::test]
    async fn test(){

        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        // let mut hasher = xxh32::Xxh32::new(0);
        // hasher.update("line subtitle".as_bytes());
        // let xxhash = hasher.digest();
        // let query_form = format!(r#"
        //     begin transaction;
        //     select persist_additional_charge('{}','{}',{});
        //     commit;
        // "#, "line subtitle",*SEED_TENANT_ID, xxhash);
        // let p = postgres_client.get().await.unwrap()
        //     .simple_query(&query_form).await.unwrap();
        // let ak = p.get(1).unwrap();
        // match ak {
        //     SimpleQueryMessage::Row(a) => {
        //         let p: Option<&str> = a.get(0);
        //         assert_that!(p).is_some()
        //             .matches(|a|**a == *crate::invoicing::line_subtitle::line_subtitle_models::tests::SEED_SUBTITLE_ID.to_string().as_str());
        //     }
        //     SimpleQueryMessage::CommandComplete(a) => {
        //         unreachable!();
        //     }
        //     _ =>{unreachable!();}
        // }
    }
}