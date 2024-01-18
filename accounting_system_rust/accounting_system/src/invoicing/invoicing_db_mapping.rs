use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct InvoicingDbMapping {}

const INVOICING_DDL_SQL: &str = include_str!("./invoicing_sql/invoicing_ddl.sql");

impl DbStructMapping for InvoicingDbMapping {
    fn table_name(&self) -> Option<&'static str> {
        None
    }

    fn get_ddl_script(&self) -> &'static str {
        INVOICING_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) -> &'static str {
        ""
    }

    fn get_seed_data_script(&self) -> &'static str {
        ""
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