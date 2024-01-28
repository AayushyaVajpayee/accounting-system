#[cfg(test)]
mod tests {
    use spectral::assert_that;
    use tokio_postgres::SimpleQueryMessage;
    use xxhash_rust::xxh32;
    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::invoicing::additional_charge::additional_charge_models::CreateAdditionalChargeRequestDbModel;
    use crate::invoicing::additional_charge::additional_charge_models::tests::a_create_additional_charge_request_db_model;
    use crate::tenant::tenant_models::SEED_TENANT_ID;


    fn convert_to_db_add_charge_input(charges: &Vec<CreateAdditionalChargeRequestDbModel>) -> String {
        let mut k: Vec<String> = Vec::new();
        for x in charges {
            let p = format!("('{}',{}::smallint,'{}',{},{})",x.line_id,x.line_no,x.line_title,x.title_xx_hash,x.rate);
            k.push(p);
        }
        format!("array[{}]::create_additional_charge_request[]",k.join(","))
    }

    #[tokio::test]
    async fn test() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let charges = vec![
            a_create_additional_charge_request_db_model(Default::default()),
            a_create_additional_charge_request_db_model(Default::default())];
        let query_form = format!(r#"
        begin transaction;
        select persist_additional_charge({},'{}','{}','{}');
        commit;
        "#,convert_to_db_add_charge_input(&charges),,*SEED_TENANT_ID,*SEED_USER_ID);
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