use std::fmt::Debug;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationRequest {
    pub page_size: u32,
    pub page_no: u32,
}


#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Debug + Serialize> {
    pub data: Vec<T>,
    pub meta: PaginationMetadata,
}

#[derive(Debug, Serialize)]
pub struct PaginationMetadata {
    pub current_page: u32,
    pub page_size: u32,
    pub total_pages: u32,
    pub total_count: u32,
}


#[cfg(test)]
mod tests {
    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};

    #[tokio::test]
    async fn verify_paginated_data_function_is_working() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let client = postgres_client.get().await.unwrap();
        client.simple_query("CREATE TABLE IF NOT EXISTS users (id smallserial PRIMARY KEY, username varchar , email varchar , password varchar);").await.unwrap();

        let mut list: Vec<String> = vec![];
        // Insert fake data into the users table
        for i in 1..11 {
            let query = format!("INSERT INTO users (id,username, email, password) VALUES (default,'user{}', 'user{}@example.com', 'password{}')", i, i, i);
            list.push(query);
        }
        let k = list.join(";");
        client.simple_query(k.as_str()).await.unwrap();

        // Call the get_paginated_data function
        let select_page_query = "SELECT * FROM users LIMIT 10 OFFSET 0";
        let select_count_query = "SELECT COUNT(*) FROM users";
        let page_size = 10;
        let query_xx_hash = "fake_xx_hash".as_bytes();
        let result = client.query("SELECT get_paginated_data($1, $2, $3, $4)", &[&select_page_query, &select_count_query, &page_size, &query_xx_hash]).await.unwrap();
        let row = result.into_iter().next().unwrap();

        // Parse the JSONB result
        let jsonb_data = row.get::<_, serde_json::Value>(0);
        let rows = jsonb_data["rows"].as_array().unwrap();
        let total_pages = jsonb_data["total_pages"].as_i64().unwrap();
        let total_count = jsonb_data["total_count"].as_i64().unwrap();

        // Assert the results
        assert_eq!(rows.len(), 10);
        assert_eq!(total_pages, 1); // Should be 1 since we have 10 users and page_size=10
        assert_eq!(total_count, 10);
    }
}