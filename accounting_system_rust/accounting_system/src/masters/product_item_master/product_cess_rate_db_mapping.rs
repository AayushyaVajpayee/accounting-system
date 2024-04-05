use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct ProductCessRateDbMapping {}

const PRODUCT_CESS_RATE_DDL_SQL: &str = "";
const PRODUCT_CESS_RATE_FUNCTIONS_AND_PROCEDURES_SQL: &str = "";
const PRODUCT_CESS_RATE_SEED_DATA: &str = include_str!("./product_item_sql/cess_tax_rate.csv");
impl DbStructMapping for ProductCessRateDbMapping {
    fn table_name(&self) -> Option<&'static str> {
        Some("cess_tax_rate")
    }

    fn get_ddl_script(&self) -> &'static str {
        PRODUCT_CESS_RATE_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) -> &'static str {
        PRODUCT_CESS_RATE_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        PRODUCT_CESS_RATE_SEED_DATA
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
