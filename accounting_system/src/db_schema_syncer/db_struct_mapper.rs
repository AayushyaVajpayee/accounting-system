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
use crate::ledger::ledgermaster::ledger_db_mapping::LedgerMasterDbMapping;
use crate::tenant::tenant_db_mapping::TenantDbMapping;

pub trait DbStructMapping {
    fn table_name(&self) -> &'static str;
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
    let whole_scrip = format!("{};{}", master_ddl, fn_and_procs);
    let mut txn = conn.transaction().await.unwrap();
    txn.simple_query(whole_scrip.as_str()).await.unwrap();

    for table in structs {
        let query = format!("copy {} from stdin with csv header", table.table_name());
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
        Box::new(UserDbMapping {}),
        Box::new(CurrencyDbMapping {}),
        Box::new(LedgerMasterDbMapping{}),
        Box::new(AccountTypeDbMapping {}),
        Box::new(AccountDbMapping {}),
    ];
    list
}


pub async fn init_db_with_seed(pool:Arc<Pool>){
    let tables = get_registered_table_mappings();
    execute_db_struct_mapping(tables,pool).await;
}