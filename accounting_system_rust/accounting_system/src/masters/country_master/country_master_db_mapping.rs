use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct CountryMasterDbMapping {}

const COUNTRY_MASTER_DDL_SQL: &str = include_str!("./country_master_sql/country_master_ddl.sql");
const COUNTRY_MASTER_FUNCTIONS_AND_PROCEDURES_SQL: &str =
    include_str!("./country_master_sql/country_master_functions_and_procedures.sql");
const COUNTRY_MASTER_SEED_CSV: &str = include_str!("./country_master_sql/country_master.csv");
impl DbStructMapping for CountryMasterDbMapping {
    fn table_name(&self) -> Option<&'static str> {
        Some("country_master")
    }

    fn get_ddl_script(&self) -> &'static str {
        COUNTRY_MASTER_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) -> &'static str {
        COUNTRY_MASTER_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        COUNTRY_MASTER_SEED_CSV
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
