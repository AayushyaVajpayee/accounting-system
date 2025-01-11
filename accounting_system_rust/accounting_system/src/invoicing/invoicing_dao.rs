use std::fmt::Write;
use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use deadpool_postgres::{GenericClient, Pool};
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::common_utils::pg_util::pg_util::ToPostgresString;
use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_json;
use crate::invoicing::invoicing_dao_models::InvoiceDb;
use crate::invoicing::invoicing_domain_models::CreateInvoiceDbResponse;

struct InvoicingDaoImpl {
    postgres_client: Arc<Pool>,
}

#[async_trait]
pub trait InvoicingDao: Send + Sync {
    async fn create_invoice(
        &self,
        invoice_db: &InvoiceDb,
    ) -> Result<CreateInvoiceDbResponse, DaoError>;
    async fn is_invoice_pdf_created(
        &self,
        tenant_id: Uuid,
        invoice_id: Uuid,
    ) -> Result<bool, DaoError>;
    async fn persist_invoice_pdf_dtl(
        &self,
        tenant_id: Uuid,
        invoice_id: Uuid,
        pdf_key: &str,
    ) -> Result<(), DaoError>;
}

pub fn get_invoicing_dao(arc: Arc<Pool>) -> Arc<dyn InvoicingDao> {
    let p = InvoicingDaoImpl {
        postgres_client: arc,
    };
    Arc::new(p)
}

#[async_trait]
impl InvoicingDao for InvoicingDaoImpl {
    async fn create_invoice(
        &self,
        invoice_db: &InvoiceDb,
    ) -> Result<CreateInvoiceDbResponse, DaoError> {
        let mut simple_query = String::with_capacity(1500);
        write!(&mut simple_query, "begin transaction;\n")?;
        write!(&mut simple_query, "select create_invoice(")?;
        invoice_db.fmt_postgres(&mut simple_query)?;
        write!(&mut simple_query, ");\n commit;")?;
        let conn = self.postgres_client.get().await?;
        let rows = conn.simple_query(simple_query.as_str()).await?;
        let value = parse_db_output_of_insert_create_and_return_json(&rows)?;
        let resp: CreateInvoiceDbResponse = serde_json::from_value(value)
            .context("could not deserialize into CreateInvoiceDbResponse")?;
        Ok(resp)
    }

    async fn is_invoice_pdf_created(
        &self,
        tenant_id: Uuid,
        invoice_id: Uuid,
    ) -> Result<bool, DaoError> {
        let conn = self.postgres_client.get().await?;
        let row = conn
            .query_one(
                "select exists(select 1 from invoice where tenant_id=$1 and id=$2
             and invoice_pdf_s3_id is not null)",
                &[&tenant_id, &invoice_id],
            )
            .await?;
        let is_created = row.get(0);
        Ok(is_created)
    }

    async fn persist_invoice_pdf_dtl(
        &self,
        tenant_id: Uuid,
        invoice_id: Uuid,
        pdf_key: &str,
    ) -> Result<(), DaoError> {
        let conn = self.postgres_client.get().await?;
        let query = format!(
            "update invoice set invoice_pdf_s3_id='{}' where id='{}' and tenant_id='{}'",
            pdf_key, invoice_id, tenant_id
        );
        let _ = conn.simple_query(query.as_str()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;
    use std::str::FromStr;
    use std::sync::Arc;

    use crate::masters::product_item_master::product_item_models::tests::a_product_item_response;
    use crate::masters::product_item_master::product_item_models::ProductItemResponse;
    use deadpool_postgres::{GenericClient, Pool};
    use itertools::Itertools;
    use rstest::rstest;
    use speculoos::assert_that;
    use speculoos::boolean::BooleanAssertions;
    use speculoos::option::OptionAssertions;
    use tokio_postgres::SimpleQueryMessage;
    use uuid::Uuid;
    use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_json_at_index;
    use crate::accounting::postgres_factory::test_utils_postgres::{
        get_dao_generic, get_postgres_conn_pool, get_postgres_image_port,
    };
    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::common_utils::pg_util::pg_util::ToPostgresString;
    use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_json;
    use crate::invoicing::invoicing_dao::{InvoicingDao, InvoicingDaoImpl};
    use crate::invoicing::invoicing_dao_models::convert_to_invoice_db;
    use crate::invoicing::invoicing_request_models::tests::{
        a_create_invoice_request, SEED_INVOICE_ID,
    };
    use crate::invoicing::invoicing_series::invoicing_series_models::tests::SEED_INVOICING_SERIES_MST_ID;
    use crate::invoicing::payment_term::payment_term_models::tests::SEED_PAYMENT_TERM_ID;
    use crate::masters::product_item_master::product_item_models::tests::SEED_PRODUCT_ITEM_ID;
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    async fn get_dao() -> InvoicingDaoImpl {
        get_dao_generic(|c| InvoicingDaoImpl { postgres_client: c }, None).await
    }
    fn get_products() -> Vec<Arc<ProductItemResponse>> {
        let mut d = a_product_item_response(Default::default());
        d.base_master_fields.id = *SEED_PRODUCT_ITEM_ID;
        vec![Arc::new(d)]
    }
    #[tokio::test]
    async fn persist_invoice_pdf_dtl() {
        let dao = get_dao().await;
        let req = a_create_invoice_request(Default::default());
        let pids = get_products();
        let req = req
            .to_create_invoice_with_all_details_included(pids)
            .unwrap();
        let p = convert_to_invoice_db(&req, 2, false, *SEED_USER_ID, *SEED_TENANT_ID).unwrap();
        let dp = dao.create_invoice(&p).await.unwrap();
        dao.persist_invoice_pdf_dtl(*SEED_TENANT_ID, dp.invoice_id, "somekey")
            .await
            .unwrap();
        let row = dao
            .postgres_client
            .get()
            .await
            .unwrap()
            .query_one(
                "select invoice_pdf_s3_id from invoice where id=$1 and tenant_id=$2",
                &[&dp.invoice_id, &*SEED_TENANT_ID],
            )
            .await
            .unwrap();
        let key: &str = row.get(0);
        assert_that!(key).is_equal_to("somekey")
    }

    #[tokio::test]
    async fn test_is_invoice_created() {
        let dao = get_dao().await;
        let req = a_create_invoice_request(Default::default());
        let pids = get_products();
        let req = req
            .to_create_invoice_with_all_details_included(pids)
            .unwrap();
        let p = convert_to_invoice_db(&req, 2, false, *SEED_USER_ID, *SEED_TENANT_ID).unwrap();
        let dp = dao.create_invoice(&p).await.unwrap();
        let _ = dao
            .postgres_client
            .get()
            .await
            .unwrap()
            .execute(
                "update invoice set invoice_pdf_s3_id=$1 where id =$2 and tenant_id=$3",
                &[&"some_pdf_s3_id", &dp.invoice_id, &*SEED_TENANT_ID],
            )
            .await
            .unwrap();
        let is_invoice_pdf_created = dao
            .is_invoice_pdf_created(*SEED_TENANT_ID, dp.invoice_id)
            .await
            .unwrap();
        assert_that!(is_invoice_pdf_created).is_true();
        let _ = dao
            .postgres_client
            .get()
            .await
            .unwrap()
            .execute(
                "update invoice set invoice_pdf_s3_id=null where id =$1 and tenant_id=$2",
                &[&dp.invoice_id, &*SEED_TENANT_ID],
            )
            .await
            .unwrap();
        let is_invoice_pdf_created = dao
            .is_invoice_pdf_created(*SEED_TENANT_ID, dp.invoice_id)
            .await
            .unwrap();
        assert_that!(is_invoice_pdf_created).is_false()
    }

    #[tokio::test]
    async fn test_create_invoice() {
        let dao = get_dao().await;
        let req = a_create_invoice_request(Default::default());
        let pids = get_products();
        let req = req
            .to_create_invoice_with_all_details_included(pids)
            .unwrap();
        let p = convert_to_invoice_db(&req, 2, false, *SEED_USER_ID, *SEED_TENANT_ID).unwrap();
        let p = dao.create_invoice(&p).await.unwrap();
        let row = dao
            .postgres_client
            .get()
            .await
            .unwrap()
            .query_opt("select id from invoice where id=$1", &[&p.invoice_id])
            .await
            .unwrap();
        assert_that!(row).is_some();
    }

    #[tokio::test]
    async fn test_persist_invoice_lines() {
        let dao = get_dao().await;
        let req = a_create_invoice_request(Default::default());
        let pids = get_products();
        let req = req
            .to_create_invoice_with_all_details_included(pids)
            .unwrap();
        let p = convert_to_invoice_db(&req, 2, false, *SEED_USER_ID, *SEED_TENANT_ID).unwrap();
        let mut input_str = String::with_capacity(1000);
        write!(&mut input_str, "call persist_invoice_lines(").unwrap();
        p.fmt_postgres(&mut input_str).unwrap();
        write!(&mut input_str, ",'{}')", *SEED_INVOICE_ID).unwrap();
        let _ = dao
            .postgres_client
            .get()
            .await
            .unwrap()
            .simple_query(&input_str)
            .await
            .unwrap();
        let line_id = &p.invoice_lines[0].line_id;
        let inv_line = dao
            .postgres_client
            .get()
            .await
            .unwrap()
            .query_opt("select id from invoice_line where id=$1", &[line_id])
            .await
            .unwrap();
        assert_that!(inv_line).is_some();
        let id: Uuid = inv_line.unwrap().get(0);
        assert_that!(id).is_equal_to(line_id);
    }

    #[tokio::test]
    async fn test_create_invoice_table_entry() {
        let dao = get_dao().await;
        let req = a_create_invoice_request(Default::default());
        let pids = get_products();
        let req = req
            .to_create_invoice_with_all_details_included(pids)
            .unwrap();
        let p = convert_to_invoice_db(&req, 2, false, *SEED_USER_ID, *SEED_TENANT_ID).unwrap();
        let mut input_str = String::with_capacity(1000);
        write!(&mut input_str, "select 1;").unwrap();
        write!(&mut input_str, "select create_invoice_table_entry(").unwrap();
        p.fmt_postgres(&mut input_str).unwrap();
        write!(&mut input_str, ",'{}');", *SEED_PAYMENT_TERM_ID).unwrap();
        let rows = dao
            .postgres_client
            .get()
            .await
            .unwrap()
            .simple_query(&input_str)
            .await
            .unwrap();
        // let po = rows.into_iter().skip(1).collect_vec();
        let value = parse_db_output_of_insert_create_and_return_json_at_index(&rows,4).unwrap().unwrap();
        let uuid = Uuid::from_str(value.get("invoice_id").unwrap().as_str().unwrap()).unwrap();
        let persisted_id = dao
            .postgres_client
            .get()
            .await
            .unwrap()
            .query_opt("select id from invoice where id=$1", &[&uuid])
            .await
            .unwrap();
        assert_that!(persisted_id).is_some();
    }

    #[rstest]
    #[case::without_padding(false, "TES1", "inv_num1", 2023)]
    #[case::without_padding_new_financial_year(false, "TES1", "inv_num2", 2999)]
    #[case::with_padding(true, "TES0000000000001", "inv_num3", 2023)]
    async fn test_create_invoice_number(
        #[case] padding: bool,
        #[case] result: String,
        #[case] dbname: String,
        #[case] fin_year: u16,
    ) {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, Some(dbname.as_str())).await;
        let dao = InvoicingDaoImpl {
            postgres_client: postgres_client.clone(),
        };
        if padding {
            let a = format!(
                "update invoicing_series_mst set zero_padded_counter=true where id ='{}'",
                *SEED_INVOICING_SERIES_MST_ID
            );
            dao.postgres_client
                .get()
                .await
                .unwrap()
                .simple_query(&a)
                .await
                .unwrap();
        }
        let id = *SEED_INVOICING_SERIES_MST_ID;
        let tenant_id = *SEED_TENANT_ID;
        let query_form = format!(
            r#"
            begin transaction;
            select create_invoice_number('{}','{}'::smallint,'{}','{}');
            commit;
        "#,
            id, fin_year, tenant_id, *SEED_USER_ID
        );
        let p = dao
            .postgres_client
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
                assert_that!(p).is_some().matches(|a| *a == result.as_str());
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
