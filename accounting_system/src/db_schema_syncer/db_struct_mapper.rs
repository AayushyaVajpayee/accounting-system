use std::sync::Arc;

use bytes::Bytes;
use deadpool_postgres::{GenericClient, Pool};
use futures_util::{SinkExt, stream};
use itertools::Itertools;
use pin_utils::pin_mut;
use postgres::fallible_iterator::FallibleIterator;
use tokio::pin;
use tokio_postgres::Error;

use crate::accounting::account::account_db_mapping::AccountDbMapping;
use crate::accounting::account::account_type::account_type_db_mapping::AccountTypeDbMapping;
use crate::accounting::currency::currency_db_mapping::CurrencyDbMapping;
use crate::accounting::user::user_db_mapping::UserDbMapping;
use crate::audit_table::audit_table_db_mapping::AuditTableDbMapping;
use crate::common_utils::common_utils_db_mapping::CommonUtilsDbMapping;
use crate::common_utils::pagination::pagination_db_mapping::PaginationDataDbMapping;
use crate::ledger::ledger_transfer_db_mapping::LedgerTransferDbMapping;
use crate::ledger::ledgermaster::ledger_db_mapping::LedgerMasterDbMapping;
use crate::masters::address_master::address_db_mapping::AddressDbMapping;
use crate::masters::city_master::city_master_db_mapping::CityMasterDbMapping;
use crate::masters::company_master::company_master_db_mapping::CompanyMasterDbMapping;
use crate::masters::company_master::company_unit_master::company_unit_db_mapping::CompanyUnitMasterDbMapping;
use crate::masters::country_master::country_master_db_mapping::CountryMasterDbMapping;
use crate::masters::pincode_master::pincode_master_db_mapping::PincodeMasterDbMapping;
use crate::masters::state_master::state_master_db_mapping::StateMasterDbMapping;
use crate::tenant::tenant_db_mapping::TenantDbMapping;

pub trait DbStructMapping {
    fn table_name(&self) -> Option<&'static str>;
    fn get_ddl_script(&self) -> &'static str; //can embed in code
    fn get_index_creation_script(&self) -> &'static str; //can embed in code
    fn get_functions_and_procedures_script(&self) -> &'static str; //can embed in code
    fn get_seed_data_script(&self) -> &'static str; //should not embed in code. but will do for now
    fn get_migration_ddl_script(&self) -> String;
    fn get_migration_functions_and_procedures_script(&self) -> String;
    fn get_migration_dml_statements_script(&self) -> String;
    fn get_migrations_index_creation_script(&self) -> String;
    fn get_migrations_seed_data_script(&self) -> String;
}

async fn execute_db_struct_mapping(structs: Vec<Box<dyn DbStructMapping>>,pool:Arc<Pool>) {
    let mut conn = pool.get().await.unwrap();
    let master_ddl = structs.iter().map(|s| s.get_ddl_script()).join(";");
    let fn_and_procs = structs
        .iter()
        .map(|s| s.get_functions_and_procedures_script())
        .join(";");
    let constraints_and_indexes = structs.iter().map(|s| s.get_index_creation_script())
        .join(";");
    let whole_scrip = format!("{};{};{}", master_ddl, fn_and_procs, constraints_and_indexes);
    let mut txn = conn.transaction().await.unwrap();
    txn.simple_query(whole_scrip.as_str()).await.unwrap();

    for table in structs {
        if table.table_name().is_none(){
            continue;
        }
        let query = format!("copy {} from stdin with csv header", table.table_name().unwrap());
        let content = async { Ok::<_, Error>(Bytes::from(table.get_seed_data_script())) };
        let stream = stream::once(content);
        let copy_in_writer = txn.copy_in(&query).await.unwrap();
        pin_mut!(copy_in_writer);
        pin!(stream);
        copy_in_writer.send_all(&mut stream).await.unwrap();
        copy_in_writer.finish().await.unwrap();
    }
    txn.commit().await.unwrap();
}


fn get_registered_table_mappings() -> Vec<Box<dyn DbStructMapping>> {
    let list: Vec<Box<dyn DbStructMapping>> = vec![
        Box::new(TenantDbMapping {}),
        Box::new(AuditTableDbMapping{}),
        Box::new(CommonUtilsDbMapping{}),
        Box::new(PaginationDataDbMapping{}),
        Box::new(UserDbMapping {}),
        Box::new(CurrencyDbMapping {}),
        Box::new(LedgerMasterDbMapping{}),
        Box::new(AccountTypeDbMapping {}),
        Box::new(AccountDbMapping {}),
        Box::new(LedgerTransferDbMapping{}),
        Box::new(CountryMasterDbMapping{}),
        Box::new(StateMasterDbMapping{}),
        Box::new(CityMasterDbMapping{}),
        Box::new(PincodeMasterDbMapping{}),
        Box::new(AddressDbMapping{}),
        Box::new(CompanyMasterDbMapping{}),
        Box::new(CompanyUnitMasterDbMapping{})
    ];
    list
}


pub async fn init_db_with_seed(pool:Arc<Pool>){
    let tables = get_registered_table_mappings();
    execute_db_struct_mapping(tables,pool).await;
}