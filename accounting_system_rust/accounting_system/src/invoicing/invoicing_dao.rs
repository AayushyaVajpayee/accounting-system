use std::sync::Arc;

use async_trait::async_trait;
use deadpool_postgres::{Pool};
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::common_utils::pg_util::pg_util::ToPostgresString;
use crate::invoicing::invoicing_dao_models::InvoiceDb;
use std::fmt::Write;
use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_uuid;

struct InvoicingDaoImpl {
    postgres_client: Arc<Pool>,
}

#[async_trait]
pub trait InvoicingDao: Send + Sync {
    async fn create_invoice(&self, invoice_db: &InvoiceDb) -> Result<Uuid, DaoError>;
}

pub fn get_invoicing_dao(arc:Arc<Pool>)->Arc<dyn InvoicingDao>{
   let p = InvoicingDaoImpl{
        postgres_client: arc,
    };
   Arc::new(p)
}

#[async_trait]
impl InvoicingDao for InvoicingDaoImpl {
    async fn create_invoice(&self, invoice_db: &InvoiceDb) -> Result<Uuid, DaoError> {
        let mut simple_query = String::with_capacity(1500);
        write!(&mut simple_query, "begin transaction;\n")?;
        write!(&mut simple_query, "select create_invoice(")?;
        invoice_db.fmt_postgres(&mut simple_query)?;
        write!(&mut simple_query, ");\n commit;")?;
        let conn = self.postgres_client.get().await?;
        let rows = conn.simple_query(simple_query.as_str()).await?;
        parse_db_output_of_insert_create_and_return_uuid(&rows)
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
    use crate::common_utils::pg_util::pg_util::ToPostgresString;
    use crate::invoicing::invoicing_dao::{InvoicingDao, InvoicingDaoImpl};
    use crate::invoicing::invoicing_dao_models::{convert_to_invoice_db};
    use crate::invoicing::invoicing_request_models::tests::{a_create_invoice_request, SEED_INVOICE_ID};
    use crate::invoicing::invoicing_series::invoicing_series_models::tests::SEED_INVOICING_SERIES_MST_ID;
    use crate::invoicing::payment_term::payment_term_models::tests::SEED_PAYMENT_TERM_ID;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;
    use std::fmt::Write;
    use std::str::FromStr;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_invoice() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let dao = InvoicingDaoImpl { postgres_client: postgres_client.clone() };
        let req = a_create_invoice_request(Default::default());
        let p = convert_to_invoice_db(&req, 2,
                                      false, *SEED_USER_ID,
                                      *SEED_TENANT_ID).unwrap();
        let p = dao.create_invoice(&p).await.unwrap();
        let row = dao.postgres_client.get().await.unwrap()
            .query_opt("select id from invoice where id=$1", &[&p])
            .await.unwrap();
        assert_that!(row).is_some();
    }

    #[tokio::test]
    async fn test_persist_invoice_lines() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let dao = InvoicingDaoImpl { postgres_client: postgres_client.clone() };
        let req = a_create_invoice_request(Default::default());
        let p = convert_to_invoice_db(&req, 2,
                                      false, *SEED_USER_ID,
                                      *SEED_TENANT_ID).unwrap();
        let mut input_str = String::with_capacity(1000);
        write!(&mut input_str, "call persist_invoice_lines(").unwrap();
        p.fmt_postgres(&mut input_str).unwrap();
        write!(&mut input_str, ",'{}')", *SEED_INVOICE_ID).unwrap();
        let _ = dao.postgres_client.get().await.unwrap()
            .simple_query(&input_str).await.unwrap();
        let line_id = &p.invoice_lines[0].line_id;
        let inv_line = dao.postgres_client.get().await.unwrap()
            .query_opt("select id from invoice_line where id=$1", &[line_id])
            .await.unwrap();
        assert_that!(inv_line).is_some();
        let id: Uuid = inv_line.unwrap().get(0);
        assert_that!(id).is_equal_to(line_id);
    }

    #[tokio::test]
    async fn test_create_invoice_table_entry() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let dao = InvoicingDaoImpl { postgres_client: postgres_client.clone() };
        let req = a_create_invoice_request(Default::default());
        let p = convert_to_invoice_db(&req, 2,
                                      false, *SEED_USER_ID,
                                      *SEED_TENANT_ID).unwrap();
        let mut input_str = String::with_capacity(1000);
        write!(&mut input_str, "select create_invoice_table_entry(").unwrap();
        p.fmt_postgres(&mut input_str).unwrap();
        write!(&mut input_str, ",'{}')", *SEED_PAYMENT_TERM_ID).unwrap();
        let rows = dao.postgres_client.get().await.unwrap()
            .simple_query(&input_str).await.unwrap();
        let id = rows.first().unwrap();
        let uuid = match id {
            SimpleQueryMessage::Row(a) => {
                Uuid::from_str(a.get(0).unwrap()).unwrap()
            }
            _ => { panic!("panic") }
        };
        let persisted_id = dao.postgres_client.get().await.unwrap()
            .query_opt("select id from invoice where id=$1", &[&uuid])
            .await.unwrap();
        assert_that!(persisted_id).is_some();
    }

    #[tokio::test]
    async fn test_create_invoice_request_rust_struct_to_db_composite_type_mapping() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let req = a_create_invoice_request(Default::default());
        let invoice_db =
            convert_to_invoice_db(&req, 2,
                                  false,
                                  *SEED_USER_ID, *SEED_TENANT_ID)
                .unwrap();
        let mut invoice_db_str = String::with_capacity(1000);

        let _ = postgres_client.get().await.unwrap()
            .query("select $1::create_invoice_request", &[&invoice_db])
            .await.unwrap();
        invoice_db.fmt_postgres(&mut invoice_db_str).unwrap();
        let m = format!("select {}::create_invoice_request", invoice_db_str);
        let _ = postgres_client.get().await.unwrap()
            .simple_query(&m).await.unwrap();
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