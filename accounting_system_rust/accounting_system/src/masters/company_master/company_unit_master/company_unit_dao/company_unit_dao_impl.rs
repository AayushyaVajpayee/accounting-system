use anyhow::{anyhow, Context};
use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::{GenericClient, Pool};
use std::sync::Arc;
use tokio_postgres::Row;
use uuid::Uuid;
use xxhash_rust::xxh32;

use crate::common_utils::dao_error::DaoError;
use crate::common_utils::db_row_conversion_utils::{convert_row_to_audit_metadata_base, convert_row_to_base_master_fields};
use crate::common_utils::pagination::pagination_utils::{PAGINATED_DATA_QUERY, PaginatedDbResponse, PaginatedResponse, PaginationMetadata};
use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_uuid;
use crate::masters::address_master::address_utils::create_address_input_for_db_function;
use crate::masters::company_master::company_master_models::gstin_no::GstinNo;
use crate::masters::company_master::company_unit_master::company_unit_dao::company_unit_dao::CompanyUnitDao;
use crate::masters::company_master::company_unit_master::company_unit_models::{CompanyUnitAddressRequest, CompanyUnitMaster, CreateCompanyUnitRequest};

const TABLE_NAME: &str = "company_unit_master";
const SELECT_FIELDS: &str = "id,entity_version_id,tenant_id,active,approval_status,remarks,company_id,address_id,gstin,created_by,updated_by,created_at,updated_at";
const QUERY_BY_ID: &str = concatcp!("select ",SELECT_FIELDS," from ",TABLE_NAME," where id=$1");

struct CompanyUnitDaoImpl {
    postgres_client: Arc<Pool>,
}

#[allow(dead_code)]
pub fn get_company_master_unit_dao(pool: Arc<Pool>) -> Arc<dyn CompanyUnitDao> {
    let dao = CompanyUnitDaoImpl {
        postgres_client: pool,
    };
    Arc::new(dao)
}


impl TryFrom<&Row> for CompanyUnitMaster {
    type Error = DaoError;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let (base_master_fields, next_ind) = convert_row_to_base_master_fields(&row)?;
        Ok(
            CompanyUnitMaster {
                base_master_fields,
                company_id: row.try_get(next_ind)?,
                address_id: row.try_get(next_ind + 1)?,
                gstin: GstinNo::new(row.try_get(next_ind + 2)?).context("error during db row conversion for gstin")?,
                audit_metadata: convert_row_to_audit_metadata_base(next_ind + 3, row)?,
            }
        )
    }
}


#[async_trait]
impl CompanyUnitDao for CompanyUnitDaoImpl {
    async fn create_company_unit(&self, request1: &CreateCompanyUnitRequest) -> Result<Uuid, DaoError> {
        let query = match &request1.address {
            CompanyUnitAddressRequest::ExistingAddress { id } => {
                format!(r#"
           begin transaction;
           select create_company_unit_master(Row('{}','{}','{}','{}','{}',{},{}::smallint,{},'{}'));
           commit;
        "#,
                        request1.idempotency_key, request1.tenant_id,
                        request1.company_id, request1.gstin_no.get_str(),
                        request1.created_by, true, 1, "null", id
                )
            }
            CompanyUnitAddressRequest::NewAddress { .. } => {
                let addr_req = request1.to_create_address_request()
                    .ok_or(anyhow!("unable to transform to CreateAddressRequest"))?;
                format!(r#"
           begin transaction;
           select create_company_unit_master(Row('{}','{}','{}','{}','{}',{},{}::smallint,{},{}));
           commit;
        "#,
                        request1.idempotency_key, request1.tenant_id,
                        request1.company_id, request1.gstin_no.get_str(),
                        request1.created_by, true, 1, create_address_input_for_db_function(&addr_req), "null"
                )
            }
        };
        let conn = self.postgres_client.get().await?;
        let rows = conn.simple_query(query.as_str()).await?;
        parse_db_output_of_insert_create_and_return_uuid(&rows)
    }


    async fn get_company_unit_by_id(&self, id: &Uuid) -> Result<Option<CompanyUnitMaster>, DaoError> {
        let query = QUERY_BY_ID;
        let cmp_unit: Option<CompanyUnitMaster> = self.postgres_client.get().await?
            .query_opt(query, &[id]).await?
            .map(|a| (&a).try_into())
            .transpose()?;
        Ok(cmp_unit)
    }

    async fn get_company_units_by_company_id(&self, company_id: &Uuid, page_no: u32, per_page: u32) -> Result<PaginatedResponse<CompanyUnitMaster>, DaoError> {
        let conn = self.postgres_client.get().await?;
        let select_rows_query = format!("select * from company_unit_master where company_id='{}' order by id asc limit {} offset {}", company_id, per_page, (page_no - 1) * per_page);
        let select_count_query = format!("select count(*) from company_unit_master where company_id='{}'", company_id);
        let mut hasher = xxh32::Xxh32::new(0);
        hasher.update("get_company_units_by_company_id".as_bytes());
        hasher.update(company_id.as_bytes());
        let hash = hasher.digest();
        let rows = conn
            .query(PAGINATED_DATA_QUERY, &[&select_rows_query, &select_count_query, &(per_page as i32), &(hash as i64)])
            .await?
            .iter()
            .map(|a| a.get::<usize, serde_json::Value>(0))
            .map(|a| {
                let p = serde_json::from_value::<PaginatedDbResponse<CompanyUnitMaster>>(a);
                p
            })
            .next()
            .transpose()
            .context("error during de-serialising row in get_company_units_by_company_id")?;
        rows.map_or_else(
            || Ok(PaginatedResponse {
                data: vec![],
                meta: PaginationMetadata {
                    current_page: page_no,
                    page_size: per_page,
                    total_pages: 0,
                    total_count: 0,
                },
            }),
            |db_page| {
                let row_can = db_page.rows;
                Ok(PaginatedResponse {
                    data: row_can,
                    meta: PaginationMetadata {
                        current_page: page_no,
                        page_size: per_page,
                        total_pages: db_page.total_pages,
                        total_count: db_page.total_count,
                    },
                })
            },
        )



    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::{OptionAssertions, VecAssertions};
    use uuid::Uuid;

    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::common_utils::dao_error::DaoError;
    use crate::masters::address_master::address_model::tests::{a_create_address_request, SEED_ADDRESS_ID};
    use crate::masters::company_master::company_master_models::company_master::tests::SEED_COMPANY_MASTER_ID;
    use crate::masters::company_master::company_master_models::gstin_no::GstinNo;
    use crate::masters::company_master::company_unit_master::company_unit_dao::company_unit_dao::CompanyUnitDao;
    use crate::masters::company_master::company_unit_master::company_unit_dao::company_unit_dao_impl::CompanyUnitDaoImpl;
    use crate::masters::company_master::company_unit_master::company_unit_models::{CompanyUnitAddressRequest, CreateCompanyUnitRequestBuilder};
    use crate::masters::company_master::company_unit_master::company_unit_models::tests::a_create_company_unit_request;

    #[tokio::test]
    async fn test_paginated_get_company_units_by_company_id() {
        let port = get_postgres_image_port().await;
        let pg_pool = get_postgres_conn_pool(port, Some("get_company_units_by_company_id")).await;
        {
            pg_pool.get().await.unwrap().simple_query("delete from company_unit_master").await.unwrap();
        }
        let company_master_dao = CompanyUnitDaoImpl { postgres_client: pg_pool };
        for _i in 0..20 {
            let k = a_create_company_unit_request(Default::default());
            company_master_dao.create_company_unit(&k).await.unwrap();//todo create batch api
        }
        let p =
            company_master_dao.get_company_units_by_company_id(&SEED_COMPANY_MASTER_ID, 1, 10)
                .await.unwrap();
        assert_that!(p.meta.current_page).is_equal_to(1);
        assert_that!(p.meta.page_size).is_equal_to(10);
        assert_that!(p.meta.total_count).is_equal_to(20);
        assert_that!(p.meta.total_pages).is_equal_to(2);
        assert_that!(p.data).has_length(10);
    }
    #[tokio::test]
    async fn should_throw_an_error_if_already_existing_company_unit_with_the_same_gstin_and_idempotency_key_is_different() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let dao = CompanyUnitDaoImpl { postgres_client: postgres_client.clone() };
        let mut create_request =
            a_create_company_unit_request(CreateCompanyUnitRequestBuilder::default());
        create_request.gstin_no = GstinNo::new("27AAAFU0696A1ZE").unwrap();
        let _created_id_1 = dao.create_company_unit(&create_request).await.unwrap();
        create_request.gstin_no = GstinNo::new("27AAAFU0696A1ZE").unwrap();
        create_request.idempotency_key = Uuid::now_v7();
        let err = dao.create_company_unit(&create_request).await.unwrap_err();
        assert!(matches!(err,DaoError::UniqueConstraintViolated {..}));
    }

    #[tokio::test]
    async fn should_return_existing_company_unit_if_it_already_exists_based_on_idempotency_key() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let dao = CompanyUnitDaoImpl { postgres_client: postgres_client.clone() };
        let create_request =
            a_create_company_unit_request(CreateCompanyUnitRequestBuilder::default());
        let created_id_1 = dao.create_company_unit(&create_request).await.unwrap();
        let created_id_2 = dao.create_company_unit(&create_request).await.unwrap();
        assert_that!(created_id_1)
            .is_equal_to(created_id_2)
    }

    #[rstest]
    #[case(CompanyUnitAddressRequest::ExistingAddress{id: * SEED_ADDRESS_ID})]
    #[case(CompanyUnitAddressRequest::NewAddress {request: a_create_address_request(Default::default()).into()})]
    async fn should_be_able_to_create_company_unit_with_existing_address_id(#[case] add_type: CompanyUnitAddressRequest) {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let dao = CompanyUnitDaoImpl { postgres_client: postgres_client.clone() };
        let mut buil = CreateCompanyUnitRequestBuilder::default();
        buil.address(add_type);
        let create_request =
            a_create_company_unit_request(buil);
        let created_id = dao.create_company_unit(&create_request).await.unwrap();
        let k = dao.get_company_unit_by_id(&created_id).await.unwrap();
        assert_that!(k).is_some()
            .map(|a| &a.base_master_fields.id)
            .is_equal_to(created_id);
    }
}