use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct PaginationDataDbMapping{}

const PAGINATION_DATA_DDL_SQL: &str = include_str!("./pagination_data_sql/pagination_data_cache_ddl.sql");
const PAGINATION_DATA_FUNCTIONS_AND_PROCEDURES_SQL:&str = include_str!("./pagination_data_sql/pagination_data_cache_functions_and_procedures.sql");
const PAGINATION_DATA_SEED_CSV:&str ="";
impl DbStructMapping for PaginationDataDbMapping{
    fn table_name(&self) -> Option<&'static str> {
        None
    }

    fn get_ddl_script(&self) -> &'static str {
        PAGINATION_DATA_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) ->  &'static str {
        PAGINATION_DATA_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        PAGINATION_DATA_SEED_CSV
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
