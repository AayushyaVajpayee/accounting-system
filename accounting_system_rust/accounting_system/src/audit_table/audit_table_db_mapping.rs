use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct AuditTableDbMapping {}

const AUDIT_TABLE_DDL_SQL: &str = include_str!("./audit_table_sql/audit_table_ddl.sql");
const AUDIT_TABLE_FUNCTIONS_AND_PROCEDURES_SQL: &str =
    include_str!("./audit_table_sql/audit_table_functions_and_procedures.sql");
const AUDIT_TABLE_SEED_CSV: &str = include_str!("./audit_table_sql/audit_table.csv");
impl DbStructMapping for AuditTableDbMapping {
    fn table_name(&self) -> Option<&'static str> {
        Some("audit_entries")
    }

    fn get_ddl_script(&self) -> &'static str {
        AUDIT_TABLE_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) -> &'static str {
        AUDIT_TABLE_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        AUDIT_TABLE_SEED_CSV
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
