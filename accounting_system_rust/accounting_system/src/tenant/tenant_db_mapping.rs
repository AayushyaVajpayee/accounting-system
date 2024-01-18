use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct TenantDbMapping{}

const TENANT_DDL_SQL: &str = include_str!("./tenant_sql/tenant_ddl.sql");
const TENANT_FUNCTIONS_AND_PROCEDURES_SQL:&str = include_str!("./tenant_sql/tenant_functions_and_procedures.sql");
const TENANT_SEED_CSV:&str =include_str!("./tenant_sql/tenant.csv");
impl DbStructMapping for TenantDbMapping{
    fn table_name(&self) -> Option<&'static str> {
        Some("tenant")
    }

    fn get_ddl_script(&self) -> &'static str {
        TENANT_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) ->  &'static str {
        TENANT_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        TENANT_SEED_CSV
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
