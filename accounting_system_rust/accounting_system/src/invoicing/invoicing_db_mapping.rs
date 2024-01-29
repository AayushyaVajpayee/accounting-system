use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct InvoicingDbMapping {}

const INVOICING_DDL_SQL: &str = include_str!("./invoicing_sql/invoicing_ddl.sql");
const INVOICING_FUNCTIONS_AND_PROCEDURES_SQL: &str = include_str!("./invoicing_sql/invoicing_functions_and_procedures.sql");
const INVOICING_SEED_DATA: &str = include_str!("./invoicing_sql/invoice.csv");
impl DbStructMapping for InvoicingDbMapping {
    fn table_name(&self) -> Option<&'static str> {
        Some("invoice")
    }

    fn get_ddl_script(&self) -> &'static str {
        INVOICING_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) -> &'static str {
        INVOICING_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        INVOICING_SEED_DATA
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