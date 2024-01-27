use std::sync::Arc;

use deadpool_postgres::Pool;

struct InvoicingDaoImpl {
    postgres_client: Arc<Pool>,
}


#[cfg(test)]
mod tests {
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::option::OptionAssertions;
    use tokio_postgres::SimpleQueryMessage;

    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::invoicing::invoicing_dao::InvoicingDaoImpl;
    use crate::invoicing::invoicing_series::invoicing_series_models::tests::SEED_INVOICING_SERIES_MST_ID;
    use crate::tenant::tenant_models::SEED_TENANT_ID;

    #[rstest]
    #[case::without_padding(false,"TES1","inv_num1")]
    #[case::with_padding(true,"TES0000000000001","inv_num2")]
    async fn test_create_invoice_number(#[case] padding:bool, #[case] result:String,#[case] dbname:String) {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, Some(dbname.as_str())).await;
        let dao = InvoicingDaoImpl { postgres_client: postgres_client.clone() };
        if padding{
            let a = format!("update invoicing_series_mst set zero_padded_counter=true where id ='{}'",*SEED_INVOICING_SERIES_MST_ID);
            dao.postgres_client.get().await.unwrap().simple_query(&a).await.unwrap();
        }
        let id = *SEED_INVOICING_SERIES_MST_ID;
        let tenant_id = *SEED_TENANT_ID;
        let query_form = format!(
            r#"
            begin transaction;
            select create_invoice_number('{}','{}'::smallint,'{}');
            commit;
        "#, id, 2024, tenant_id);
        let p = dao.postgres_client.get()
            .await
            .unwrap()
            .simple_query(&query_form)
            .await
            .unwrap();
        let ak =p.get(1).unwrap();
        match ak {
            SimpleQueryMessage::Row(a) => {
               let p: Option<&str> = a.get(0);
               assert_that!(p).is_some()
                   .matches(|a|*a==result.as_str());
            }
            SimpleQueryMessage::CommandComplete(a) => {
                unreachable!();
            }
            _ =>{unreachable!();}
        }

        println!("{}", query_form);
    }
}