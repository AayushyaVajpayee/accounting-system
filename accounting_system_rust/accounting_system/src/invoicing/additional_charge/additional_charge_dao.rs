#[cfg(test)]
mod tests {
    use log::kv::Source;
    use spectral::assert_that;
    use spectral::prelude::OptionAssertions;

    use crate::accounting::postgres_factory::test_utils_postgres::{get_dao_generic, get_postgres_conn_pool, get_postgres_image_port};
    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::invoicing::additional_charge::additional_charge_models::CreateAdditionalChargeRequestDbModel;
    use crate::invoicing::additional_charge::additional_charge_models::tests::a_create_additional_charge_request_db_model;
    use crate::invoicing::invoicing_request_models::tests::SEED_INVOICE_ID;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

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
        let postgres_client= get_dao_generic(|a|a,None).await;
        let charges = vec![
            a_create_additional_charge_request_db_model(Default::default()),
       /*     a_create_additional_charge_request_db_model(Default::default())*/];
        let query_form = format!(r#"
        begin transaction;
        call persist_additional_charge({},'{}','{}','{}');
        commit;
        "#,convert_to_db_add_charge_input(&charges),*SEED_INVOICE_ID,*SEED_TENANT_ID,*SEED_USER_ID);
        let _ = postgres_client.get()
            .await.unwrap()
            .simple_query(&query_form)
            .await.unwrap();
        let id = charges.get(0).unwrap().line_id;
        let asd = postgres_client.get()
            .await.unwrap()
            .query_opt("select * from additional_charge where id =$1",
                       &[&id]).await.unwrap();

        assert_that!(asd).is_some();

    }
}