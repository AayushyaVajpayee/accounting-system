use std::sync::Arc;

use async_trait::async_trait;
use const_format::concatcp;
use deadpool_postgres::{GenericClient, Pool};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::common_utils::dao_error::DaoError;
use crate::common_utils::db_row_conversion_utils::{convert_row_to_audit_metadata_base, convert_row_to_base_master_fields};
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
        todo!()
    }
}