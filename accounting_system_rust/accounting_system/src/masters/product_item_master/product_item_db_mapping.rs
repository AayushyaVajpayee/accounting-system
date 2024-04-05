use crate::db_schema_syncer::db_struct_mapper::DbStructMapping;

pub struct ProductItemDbMapping {}

const PRODUCT_ITEM_DDL_SQL: &str = include_str!("./product_item_sql/product_item_ddl.sql");
const PRODUCT_ITEM_FUNCTIONS_AND_PROCEDURES_SQL: &str =
    include_str!("./product_item_sql/product_item_functions_and_procedures.sql");
const PRODUCT_ITEM_SEED_DATA: &str = include_str!("./product_item_sql/product_item.csv");
impl DbStructMapping for ProductItemDbMapping {
    fn table_name(&self) -> Option<&'static str> {
        Some("product_item")
    }

    fn get_ddl_script(&self) -> &'static str {
        PRODUCT_ITEM_DDL_SQL
    }

    fn get_index_creation_script(&self) -> &'static str {
        ""
    }

    fn get_functions_and_procedures_script(&self) -> &'static str {
        PRODUCT_ITEM_FUNCTIONS_AND_PROCEDURES_SQL
    }

    fn get_seed_data_script(&self) -> &'static str {
        PRODUCT_ITEM_SEED_DATA
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
