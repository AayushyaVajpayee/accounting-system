use std::sync::Arc;

use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::{GenericClient, Pool};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::common_utils::db_row_conversion_utils::{convert_row_to_audit_metadata_base, convert_row_to_base_master_fields};
use crate::common_utils::utils::parse_db_output_of_insert_create_and_return_uuid;
use crate::masters::address_master::address_model::{Address, AddressLine, CreateAddressRequest};

#[async_trait]
pub trait AddressDao: Send + Sync {
    async fn get_address_by_id(&self, address_id: &Uuid) -> Result<Option<Address>, DaoError>;
    async fn create_address(&self, request: &CreateAddressRequest) -> Result<Uuid, DaoError>;
}

struct AddressDaoImpl {
    postgres_client: Arc<Pool>,
}


impl TryFrom<Row> for Address {
    type Error = DaoError;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        let (base_master_fields, next_ind) = convert_row_to_base_master_fields(&row)?;
        let addr = Address {
            base_master_fields,
            pincode_id: row.try_get(next_ind)?,
            city_id: row.try_get(next_ind + 1)?,
            state_id: row.try_get(next_ind + 2)?,
            country_id: row.try_get(next_ind + 3)?,
            line_1: AddressLine::new(row.try_get(next_ind + 4)?)?,
            line_2: AddressLine::new_nullable(row.try_get(next_ind + 5)?)?,
            landmark: AddressLine::new_nullable(row.try_get(next_ind + 6)?)?,
            audit_metadata: convert_row_to_audit_metadata_base(next_ind + 7, &row)?,
        };
        Ok(addr)
    }
}


pub fn get_address_dao(client: Arc<Pool>) -> Arc<dyn AddressDao> {
    let ad = AddressDaoImpl {
        postgres_client: client
    };
    Arc::new(ad)
}


const TABLE_NAME: &str = "address";
const SELECT_FIELDS: &str = "id,entity_version_id,tenant_id,active,approval_status,remarks,pincode_id,city_id,state_id,country,line_1,line_2,landmark,created_by,updated_by,created_at,updated_at";
const QUERY_BY_ID: &str = concatcp!("select ",SELECT_FIELDS," from ",TABLE_NAME," where id=$1");

#[async_trait]
impl AddressDao for AddressDaoImpl {
    async fn get_address_by_id(&self, address_id: &Uuid) -> Result<Option<Address>, DaoError> {
        let query = QUERY_BY_ID;
        let addr: Option<Address> = self.postgres_client.get().await?
            .query_opt(query, &[&address_id]).await?
            .map(|a| a.try_into())
            .transpose()?;
        Ok(addr)
    }

    async fn create_address(&self, request: &CreateAddressRequest) -> Result<Uuid, DaoError> {
        let simple_query = format!(
            r#"
            begin transaction;
            select create_address(Row('{}','{}','{}',{},{},'{}','{}','{}','{}','{}',{}::smallint));
            commit;
            "#,
            request.idempotence_key,
            request.tenant_id,
            request.line_1,
            request.line_2.as_ref().map(|a| format!("'{}'", a))
                .unwrap_or_else(|| "null".to_string()),
            request.landmark.as_ref().map(|a| format!("'{}'", a))
                .unwrap_or_else(|| "null".to_string()),
            request.city_id,
            request.state_id,
            request.country_id,
            request.pincode_id,
            request.created_by,
            1
        );
        let conn = self.postgres_client.get().await?;
        let rows = conn.simple_query(simple_query.as_str()).await?;
        parse_db_output_of_insert_create_and_return_uuid(&rows)
    }
}


#[cfg(test)]
mod tests {
    use spectral::assert_that;
    use spectral::prelude::OptionAssertions;

    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::masters::address_master::address_dao::{AddressDao, AddressDaoImpl};
    use crate::masters::address_master::address_model::CreateAddressRequestBuilder;
    use crate::masters::address_master::address_model::tests::a_create_address_request;

    #[tokio::test]
    async fn test_insert_and_get_address() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port, None).await;
        let dao = AddressDaoImpl { postgres_client: postgres_client.clone() };
        let address = a_create_address_request(CreateAddressRequestBuilder::default());
        let id = dao.create_address(&address).await.unwrap();
        let k = dao.get_address_by_id(&id).await.unwrap();
        assert_that!(k).is_some()
            .map(|a| &a.base_master_fields.id)
            .is_equal_to(id);
    }
}
