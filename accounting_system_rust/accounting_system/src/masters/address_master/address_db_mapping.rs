use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct AddressDbMapping {}

const ADDRESS_DDL_SQL: &str = include_str!("./address_sql/address_ddl.sql");
const ADDRESS_FUNCTIONS_AND_PROCEDURES_SQL: &str =
    include_str!("./address_sql/address_functions_and_procedures.sql");
const ADDRESS_SEED_CSV: &str = include_str!("./address_sql/address.csv");
impl DbStructMapping for AddressDbMapping {
    fn table_name(&self) -> Option<&'static str> {
        Some("address")
    }

    fn get_ddl_script(&self) -> &'static str {
        ADDRESS_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) -> &'static str {
        ADDRESS_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        ADDRESS_SEED_CSV
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
