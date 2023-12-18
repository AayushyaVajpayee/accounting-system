use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct LedgerTransferDbMapping{}

const LEDGER_TRANSFER_DDL_SQL: &str = include_str!("./ledger_transfer_sql/ledger_transfer_ddl.sql");
const LEDGER_TRANSFER_FUNCTIONS_AND_PROCEDURES_SQL:&str = include_str!("./ledger_transfer_sql/ledger_transfer_functions_and_procedures.sql");
const LEDGER_TRANSFER_SEED_CSV:&str =include_str!("./ledger_transfer_sql/ledger_transfer.csv");
impl DbStructMapping for LedgerTransferDbMapping{
    fn table_name(&self) -> &'static str {
        "transfer"
    }

    fn get_ddl_script(&self) -> &'static str {
        LEDGER_TRANSFER_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) ->  &'static str {
        LEDGER_TRANSFER_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        LEDGER_TRANSFER_SEED_CSV
    }

    fn get_migration_ddl_script(&self) -> String {
        todo!()
    }

    fn get_migration_functions_and_procedures_script(&self) -> String {
        todo!()
    }

    fn get_migration_dml_statements_script(&self) -> String {
        todo!()
    }

    fn get_migrations_index_creation_script(&self) -> String {
        todo!()
    }

    fn get_migrations_seed_data_script(&self) -> String {
        todo!()
    }
}
