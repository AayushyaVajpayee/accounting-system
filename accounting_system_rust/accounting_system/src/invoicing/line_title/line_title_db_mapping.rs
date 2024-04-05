use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct LineTitleDbMapping {}

const LINE_TITLE_DDL_SQL: &str = include_str!("./line_title_sql/line_title_ddl.sql");
const LINE_TITLE_FUNCTIONS_AND_PROCEDURES_SQL: &str =
    include_str!("./line_title_sql/line_title_function_and_procedures.sql");
const LINE_TITLE_SEED_DATA: &str = include_str!("./line_title_sql/line_title.csv");
impl DbStructMapping for LineTitleDbMapping {
    fn table_name(&self) -> Option<&'static str> {
        Some("line_title")
    }

    fn get_ddl_script(&self) -> &'static str {
        LINE_TITLE_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) -> &'static str {
        LINE_TITLE_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        LINE_TITLE_SEED_DATA
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
