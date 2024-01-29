use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct InvoiceTemplateDbMapping {}

const INVOICE_TEMPLATE_DDL_SQL: &str = include_str!("./invoice_template_sql/invoice_template_ddl.sql");
const INVOICE_TEMPLATE_FUNCTIONS_AND_PROCEDURES_SQL: &str = include_str!("./invoice_template_sql/invoice_template_functions_and_procedures.sql");
const INVOICE_TEMPLATE_SEED_DATA: &str = include_str!("./invoice_template_sql/invoice_template.csv");
impl DbStructMapping for InvoiceTemplateDbMapping {
    fn table_name(&self) -> Option<&'static str> {
        Some("invoice_template")
    }

    fn get_ddl_script(&self) -> &'static str {
        INVOICE_TEMPLATE_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) -> &'static str {
        INVOICE_TEMPLATE_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        INVOICE_TEMPLATE_SEED_DATA
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