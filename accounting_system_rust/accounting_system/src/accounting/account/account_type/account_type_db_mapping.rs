use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct AccountTypeDbMapping {}

const ACCOUNT_TYPE_DDL_SQL: &str = include_str!("./account_type_sql/account_type_ddl.sql");
const ACCOUNT_TYPE_FUNCTIONS_AND_PROCEDURES_SQL: &str =
    include_str!("./account_type_sql/account_type_functions_and_procedures.sql");
const ACCOUNT_TYPE_SEED_CSV: &str = include_str!("./account_type_sql/account_type_seed.csv");
impl DbStructMapping for AccountTypeDbMapping {
    fn table_name(&self) -> Option<&'static str> {
        Some("account_type_master")
    }

    fn get_ddl_script(&self) -> &'static str {
        ACCOUNT_TYPE_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) -> &'static str {
        ACCOUNT_TYPE_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        ACCOUNT_TYPE_SEED_CSV
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
