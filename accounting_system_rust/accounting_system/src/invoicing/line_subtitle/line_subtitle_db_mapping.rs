use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct LineSubtitleDbMapping {}

const LINE_SUBTITLE_DDL_SQL: &str = include_str!("./line_subtitle_sql/line_subtitle_ddl.sql");
const LINE_SUBTITLE_FUNCTIONS_AND_PROCEDURES_SQL: &str =
    include_str!("./line_subtitle_sql/line_subtitle_functions_and_procedures.sql");
const LINE_SUBTITLE_SEED_DATA: &str = include_str!("./line_subtitle_sql/line_subtitle.csv");
impl DbStructMapping for LineSubtitleDbMapping {
    fn table_name(&self) -> Option<&'static str> {
        Some("line_subtitle")
    }

    fn get_ddl_script(&self) -> &'static str {
        LINE_SUBTITLE_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) -> &'static str {
        LINE_SUBTITLE_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        LINE_SUBTITLE_SEED_DATA
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
