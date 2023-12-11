use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use itertools::Itertools;
use tracing::instrument;
use uuid::Uuid;
use xxhash_rust::xxh32;

use crate::common_utils::dao_error::DaoError;
use crate::common_utils::pagination::pagination_utils::{PaginatedResponse, PaginationMetadata};
use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_uuid;
use crate::masters::company_master::company_master_models::company_master::CompanyMaster;
use crate::masters::company_master::company_master_models::master_updation_remarks::MasterUpdationRemarks;
use crate::masters::company_master::dao::dao_trait::CompanyMasterDao;
use crate::masters::company_master::dao::models::{CompanyMasterSql, PaginatedDbResponse};
use crate::masters::company_master::dao::queries_and_constants::{GET_ALL_FOR_TENANT, GET_BY_ID, SOFT_DELETE};

struct CompanyMasterDaoPostgresImpl {
    postgres_client: Arc<Pool>,
}

pub fn get_company_master_dao(pool: Arc<Pool>) -> Arc<dyn CompanyMasterDao> {
    let dao = CompanyMasterDaoPostgresImpl {
        postgres_client: pool,
    };
    Arc::new(dao)
}


#[async_trait]
impl CompanyMasterDao for CompanyMasterDaoPostgresImpl {
    async fn get_company_by_id(
        &self,
        tenant_id: Uuid,
        company_id: Uuid,
    ) -> Result<Option<CompanyMaster>, DaoError> {
        let conn = self.postgres_client.get().await?;
        let p: Option<CompanyMaster> = conn
            .query(GET_BY_ID, &[&company_id, &tenant_id])
            .await?
            .iter()
            .map(|a| a.try_into())
            .next()
            .transpose()?;
        Ok(p)
    }

    async fn get_all_companies_for_tenant(
        &self,
        tenant_id: &Uuid,
        page_no: u32,
        per_page: u32,
    ) -> Result<PaginatedResponse<CompanyMaster>, DaoError> {
        let conn = self.postgres_client.get().await?;
        let select_rows_query = format!("select * from company_master where tenant_id='{}' order by id asc limit {} offset {}", tenant_id, per_page, (page_no - 1) * per_page);
        let select_count_query = format!("select count(*) from company_master where tenant_id='{}'", tenant_id);
        let mut hasher = xxh32::Xxh32::new(0);
        hasher.update("get_companies_for_tenant_id".as_bytes());
        hasher.update(tenant_id.as_bytes());
        let hash = hasher.digest();
        let rows = conn
            .query(GET_ALL_FOR_TENANT, &[&select_rows_query, &select_count_query, &(per_page as i32), &(hash as i64)])
            .await?
            .iter()
            .map(|a| a.get::<usize, serde_json::Value>(0))
            .map(|a| {
                let p = serde_json::from_value::<PaginatedDbResponse<CompanyMasterSql>>(a);
                p
            })
            .next()
            .transpose()
            .context("error during de-serialising row in get_all_companies_for_tenant")?;
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
                let row_can: Result<Vec<CompanyMaster>, DaoError> = db_page.rows.into_iter().map(|a| a.try_into()).collect();
                Ok(PaginatedResponse {
                    data: row_can?,
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
    #[instrument(skip(self))]
    async fn create_new_company_for_tenant(&self, entity: &CompanyMaster, idempotence_key: &Uuid) -> Result<Uuid, DaoError> {
        let simple_query = format!(r#"
        begin transaction;
        select create_company_master(Row('{}',{},'{}',{},{}::smallint,{},'{}','{}','{}','{}',{},{}),'{}');
        commit;
        "#,
                                   entity.base_master_fields.id,
                                   entity.base_master_fields.entity_version_id,
                                   entity.base_master_fields.tenant_id,
                                   entity.base_master_fields.active,
                                   entity.base_master_fields.approval_status as i16,
                                   entity.base_master_fields.remarks.as_ref()
                                       .map(|a| format!("'{}'", a.get_str()))
                                       .unwrap_or_else(|| "null".to_string()),
                                   entity.name.get_str(),
                                   entity.cin.get_str(),
                                   entity.audit_metadata.created_by,
                                   entity.audit_metadata.updated_by,
                                   entity.audit_metadata.created_at,
                                   entity.audit_metadata.updated_at,
                                   idempotence_key
        );
        let conn = self.postgres_client.get().await?;
        let rows = conn.simple_query(simple_query.as_str()).await?;
        parse_db_output_of_insert_create_and_return_uuid(&rows)
    }

    async fn soft_delete_company_for_tenant(
        &self,
        tenant_id: Uuid,
        company_id: Uuid,
        entity_version_id: i32,
        remarks: &MasterUpdationRemarks,
        updated_by: Uuid,
    ) -> Result<u64, DaoError> {
        let conn = self.postgres_client.get().await?;

        let updated_rows = conn
            .execute(
                SOFT_DELETE,
                &[
                    &company_id,
                    &tenant_id,
                    &entity_version_id,
                    &remarks.get_str(),
                    &updated_by,
                ],
            )
            .await?;
        Ok(updated_rows)
    }
}

#[cfg(test)]
mod tests {
    use spectral::assert_that;
    use spectral::prelude::OptionAssertions;
    use spectral::vec::VecAssertions;
    use uuid::Uuid;

    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::audit_table::audit_service::get_audit_service_for_tests;
    use crate::masters::company_master::company_master_models::base_master_fields::tests::{a_base_master_field, BaseMasterFieldsTestDataBuilder};
    use crate::masters::company_master::company_master_models::company_master::tests::{a_company_master, CompanyMasterTestDataBuilder};
    use crate::masters::company_master::company_master_models::company_name::CompanyName;
    use crate::masters::company_master::company_master_models::master_status_enum::MasterStatusEnum::Approved;
    use crate::masters::company_master::company_master_models::master_updation_remarks::MasterUpdationRemarks;
    use crate::masters::company_master::company_master_request_response::tests::a_create_company_request;
    use crate::masters::company_master::dao::dao_trait::CompanyMasterDao;
    use crate::masters::company_master::dao::dao_trait_impl::CompanyMasterDaoPostgresImpl;
    use crate::masters::company_master::dao::queries_and_constants::TABLE_NAME;
    use crate::tenant::tenant_models::SEED_TENANT_ID;

    #[tokio::test]
    async fn test_paginated_get_all_companies_for_tenant() {
        let port = get_postgres_image_port().await;
        let pg_pool = get_postgres_conn_pool(port, Some("get_all_companies_for_tenant")).await;
        let company_master_dao = CompanyMasterDaoPostgresImpl { postgres_client: pg_pool };
        for i in 0..20 {
            let k = a_create_company_request(Default::default());
            let p = k.to_company_master().unwrap();
            company_master_dao.create_new_company_for_tenant(&p, &k.idempotence_key).await.unwrap();//todo create batch api
        }
        let p = company_master_dao.get_all_companies_for_tenant(&SEED_TENANT_ID, 1, 10)
            .await.unwrap();
        assert_that!(p.meta.current_page).is_equal_to(1);
        assert_that!(p.meta.page_size).is_equal_to(10);
        assert_that!(p.meta.total_count).is_equal_to(20);
        assert_that!(p.meta.total_pages).is_equal_to(2);
        assert_that!(p.data).has_length(10);
    }

    #[tokio::test]
    async fn test_insert_and_get_for_company_master() {
        //todo figure out how to implement idempotency in inserts and how to test them
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let dao = CompanyMasterDaoPostgresImpl { postgres_client: postgres_client.clone() };
        let company_master = a_company_master(CompanyMasterTestDataBuilder {
            base_master_fields: Some(a_base_master_field(BaseMasterFieldsTestDataBuilder {
                approval_status: Some(Approved),
                ..Default::default()
            })),
            ..Default::default()
        });
        let idempotence_key = Uuid::now_v7();
        let id = dao.create_new_company_for_tenant(&company_master, &idempotence_key)
            .await
            .unwrap();
        let k = dao
            .get_company_by_id(
                company_master.base_master_fields.tenant_id,
                id,
            )
            .await
            .unwrap();
        println!("{:?}", k);
        assert_that!(k)
            .is_some()
            .map(|a| &a.base_master_fields.id)
            .is_equal_to(id);
    }


    #[tokio::test]
    async fn should_create_company_mst_when_only_1_new_request() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let company_request = a_company_master(Default::default());
        let company_mst_dao = CompanyMasterDaoPostgresImpl { postgres_client: postgres_client.clone() };
        let idempotence_key = Uuid::now_v7();
        let id = company_mst_dao.create_new_company_for_tenant(&company_request, &idempotence_key).await.unwrap();
        let company_mst = company_mst_dao.get_company_by_id(company_request.base_master_fields.tenant_id, id).await.unwrap();
        assert_that!(company_mst).is_some();
    }

    #[tokio::test]
    async fn should_return_existing_company_mst_when_idempotency_key_is_same_as_earlier_completed_request() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let name = "testing";
        let company_mst = a_company_master(CompanyMasterTestDataBuilder { name: Some(CompanyName::new(name).unwrap()), ..Default::default() });
        let company_dao = CompanyMasterDaoPostgresImpl { postgres_client: postgres_client.clone() };
        let idempotence_key = Uuid::now_v7();
        let id = company_dao.create_new_company_for_tenant(&company_mst, &idempotence_key).await.unwrap();
        let id2 = company_dao.create_new_company_for_tenant(&company_mst, &idempotence_key).await.unwrap();
        assert_that!(&id).is_equal_to(id2);
        let number_of_company_mst_created: i64 = postgres_client
            .get()
            .await
            .unwrap()
            .query(
                "select count(id) from company_master where name=$1",
                &[&name],
            )
            .await
            .unwrap()
            .iter()
            .map(|a| a.get(0))
            .next()
            .unwrap();
        assert_that!(number_of_company_mst_created).is_equal_to(1)
        ;
    }

    #[tokio::test]
    async fn test_soft_delete_for_company_master() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let dao = CompanyMasterDaoPostgresImpl { postgres_client: postgres_client.clone() };
        let company_master = a_company_master(CompanyMasterTestDataBuilder {
            base_master_fields: Some(a_base_master_field(BaseMasterFieldsTestDataBuilder {
                approval_status: Some(Approved),
                ..Default::default()
            })),
            ..Default::default()
        });
        let idempotence_key = Uuid::now_v7();
        let id = dao.create_new_company_for_tenant(&company_master, &idempotence_key)
            .await
            .unwrap();
        let p = dao
            .get_company_by_id(
                company_master.base_master_fields.tenant_id,
                id,
            )
            .await
            .unwrap();
        let updated_rows = dao
            .soft_delete_company_for_tenant(
                company_master.base_master_fields.tenant_id,
                id,
                p.unwrap().base_master_fields.entity_version_id,
                &MasterUpdationRemarks::new("unit testing delete function").unwrap(),
                *SEED_USER_ID,
            )
            .await
            .unwrap();
        assert_that!(updated_rows).is_equal_to(1);
        let k = get_audit_service_for_tests(postgres_client);
        let ppp = k
            .get_audit_logs_for_id_and_table(id, TABLE_NAME)
            .await;
        assert_that!(ppp).has_length(1);
    }
}