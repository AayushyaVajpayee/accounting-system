use crate::tenant::tenant_db_mapping::TenantDbMapping;

pub trait DbStructMapping {
    fn table_name(&self) -> &'static str;
    fn get_ddl_script(&self) -> &'static str; //can embed in code
    fn get_index_creation_script(&self) -> &'static str; //can embed in code
    fn get_functions_and_procedures_script(&self) -> &'static str; //can embed in code
    fn get_seed_data_script(&self) -> &'static str; //should not embed in code. but will do for now
    fn get_migration_ddl_script(&self) -> String;
    fn get_migration_functions_and_procedures_script(&self) -> String;
    fn get_migration_dml_statements_script(&self) -> String;
    fn get_migrations_index_creation_script(&self) -> String;
    fn get_migrations_seed_data_script(&self) -> String;
}

async fn execute_db_struct_mapping(structs: Vec<impl DbStructMapping>) {
    // let master_script =
    //     structs.iter()
    //         .map(|a|a.get_ddl_script())
    //         .join("\n");
    //   let p = get_postgres_conn_pool();
    //   let mut conn=p.get().await.unwrap();
    //   let mut txn = conn.transaction().await.unwrap();
    //   let j=txn.simple_query(master_script.as_str()).await.unwrap();
    //   let master_script=structs.iter()
    //       .map(|a|a.get_functions_and_procedures_script())
    //       .join("\n");
    //   let ppp=txn.simple_query(master_script.as_str()).await.unwrap();
    //   let j = txn.simple_query
}

fn get_registered_table_mappings() -> Vec<Box<dyn DbStructMapping>> {
    let list: Vec<Box<dyn DbStructMapping>> = vec![
        Box::new(TenantDbMapping {}),

    ];
    list
}
