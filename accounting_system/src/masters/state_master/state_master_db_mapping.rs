use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct StateMasterDbMapping{}

const STATE_MASTER_DDL_SQL: &str = include_str!("./state_master_sql/state_master_ddl.sql");
const STATE_MASTER_FUNCTIONS_AND_PROCEDURES_SQL:&str = include_str!("./state_master_sql/state_master_functions_and_procedures.sql");
const STATE_MASTER_SEED_CSV:&str =include_str!("./state_master_sql/state_master.csv");
impl DbStructMapping for StateMasterDbMapping{
    fn table_name(&self) -> Option<&'static str> {
        Some("state_master")
    }

    fn get_ddl_script(&self) -> &'static str {
        STATE_MASTER_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) ->  &'static str {
        STATE_MASTER_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        STATE_MASTER_SEED_CSV
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
