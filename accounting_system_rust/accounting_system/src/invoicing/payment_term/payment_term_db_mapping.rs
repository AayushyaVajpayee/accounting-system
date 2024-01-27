use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct PaymentTermDbMapping {}

const PAYMENT_TERM_DDL_SQL: &str = include_str!("./payment_term_sql/payment_term_ddl.sql");
const PAYMENT_TERM_FUNCTIONS_AND_PROCEDURES_SQL: &str = include_str!("./payment_term_sql/payment_term_function_and_procedures.sql");
const PAYMENT_TERM_SEED_DATA:&str =include_str!("./payment_term_sql/payment_term.csv");
impl DbStructMapping for PaymentTermDbMapping {
    fn table_name(&self) -> Option<&'static str> {
        Some("payment_term")
    }

    fn get_ddl_script(&self) -> &'static str {
        PAYMENT_TERM_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) -> &'static str {
        PAYMENT_TERM_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        PAYMENT_TERM_SEED_DATA
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