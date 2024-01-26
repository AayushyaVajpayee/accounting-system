use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct InvoicingSeriesCounterDbMapping {}


const INVOICING_SERIES_COUNTER_SEED_DATA:&str = include_str!("./invoicing_series_sql/invoicing_series_counter.csv");

impl DbStructMapping for InvoicingSeriesCounterDbMapping {
    fn table_name(&self) -> Option<&'static str> {
        Some("invoice_series_counter")
    }

    fn get_ddl_script(&self) -> &'static str {
        ""
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) -> &'static str {
        ""
    }

    fn get_seed_data_script(&self) -> &'static str {
        INVOICING_SERIES_COUNTER_SEED_DATA
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