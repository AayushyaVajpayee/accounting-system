use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct AdditionalChargeDbMapping {}

const ADDITIONAL_CHARGE_DDL_SQL: &str = include_str!("./additional_charge_sql/additional_charge_ddl.sql");
const ADDITIONAL_CHARGE_FUNCTIONS_AND_PROCEDURES_SQL: &str = include_str!("./additional_charge_sql/additional_charge_procedures_and_functions.sql");

const ADDITIONAL_CHARGE_SEED_SCRIPT:&str = include_str!("./additional_charge_sql/additional_charge.csv");
impl DbStructMapping for AdditionalChargeDbMapping {
    fn table_name(&self) -> Option<&'static str> {
        Some("additional_charge")
    }

    fn get_ddl_script(&self) -> &'static str {
        ADDITIONAL_CHARGE_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) -> &'static str {
        ADDITIONAL_CHARGE_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        ADDITIONAL_CHARGE_SEED_SCRIPT
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