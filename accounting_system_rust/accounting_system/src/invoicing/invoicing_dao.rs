use std::sync::Arc;

use async_trait::async_trait;
use deadpool_postgres::{GenericClient, Pool};
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::invoicing::invoicing_dao_models::InvoiceDb;

struct InvoicingDaoImpl {
    postgres_client: Arc<Pool>,
}

#[async_trait]
pub trait InvoicingDao: Send + Sync {
    async fn create_invoice(&self, invoice_db: &InvoiceDb) -> Result<Uuid, DaoError>;
}

#[async_trait]
impl InvoicingDao for InvoicingDaoImpl {
    async fn create_invoice(&self, invoice_db: &InvoiceDb) -> Result<Uuid, DaoError> {
        // self.postgres_client.get().await?
        //     .query("select $1::create_additiona")
        let p = self.postgres_client.get().await?
            .prepare_typed_cached("",&[])
            // .query("select $1::create_invoice_request", &[&invoice_db])
            .await?;
        self.postgres_client.get().await?.query(&p,&[]).await.unwrap();
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use deadpool_postgres::GenericClient;
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::option::OptionAssertions;
    use tokio_postgres::SimpleQueryMessage;

    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::invoicing::invoicing_dao::InvoicingDaoImpl;
    use crate::invoicing::invoicing_dao_models::{AdditionalChargeDb, convert_to_invoice_db, InvoiceLineDb, PaymentTermsDb};
    use crate::invoicing::invoicing_request_models::tests::a_create_invoice_request;
    use crate::invoicing::invoicing_series::invoicing_series_models::tests::SEED_INVOICING_SERIES_MST_ID;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    #[tokio::test]
    async fn test_create_invoice_request_rust_struct_to_db_composite_type_mapping() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let req =a_create_invoice_request(Default::default());
        let invoice_db =
            convert_to_invoice_db(&req,2,
                                  false,
                                  *SEED_USER_ID,*SEED_TENANT_ID)
                .unwrap();
        let p= postgres_client.get().await.unwrap()
            .query("select $1::create_invoice_request", &[&invoice_db])
            .await.unwrap();
    }

    #[rstest]
    #[case::without_padding(false, "TES1", "inv_num1")]
    #[case::with_padding(true, "TES0000000000001", "inv_num2")]
    async fn test_create_invoice_number(#[case] padding: bool, #[case] result: String, #[case] dbname: String) {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, Some(dbname.as_str())).await;
        let dao = InvoicingDaoImpl { postgres_client: postgres_client.clone() };
        if padding {
            let a = format!("update invoicing_series_mst set zero_padded_counter=true where id ='{}'", *SEED_INVOICING_SERIES_MST_ID);
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
        let ak = p.get(1).unwrap();
        match ak {
            SimpleQueryMessage::Row(a) => {
                let p: Option<&str> = a.get(0);
                assert_that!(p).is_some()
                    .matches(|a| *a == result.as_str());
            }
            SimpleQueryMessage::CommandComplete(_) => {
                unreachable!();
            }
            _ => { unreachable!(); }
        }

        println!("{}", query_form);
    }
}