use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct BusinessEntityDbMapping {}

const BUSINESS_ENTITY_DDL_SQL: &str =
    include_str!("./business_entity_master_sql/business_entity_ddl.sql");
const BUSINESS_ENTITY_FUNCTIONS_AND_PROCEDURES_SQL: &str =
    include_str!("./business_entity_master_sql/business_entity_functions_and_procedures.sql");
const BUSINESS_ENTITY_SEED_CSV: &str =
    include_str!("./business_entity_master_sql/business_entity.csv");

impl DbStructMapping for BusinessEntityDbMapping {
    fn table_name(&self) -> Option<&'static str> {
        Some("business_entity")
    }

    fn get_ddl_script(&self) -> &'static str {
        BUSINESS_ENTITY_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) -> &'static str {
        BUSINESS_ENTITY_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        BUSINESS_ENTITY_SEED_CSV
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
