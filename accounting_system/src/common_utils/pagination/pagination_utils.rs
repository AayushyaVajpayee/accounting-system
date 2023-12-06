use std::collections::HashMap;
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


pub fn generate_api_link_header(base_url: &str, page: u32, per_page: u32, total_count: u32) -> String {
    let links = generate_links(base_url, page, per_page, total_count);
    let link_header = links.iter().map(|(&rel, url)| format!("{}: <{}>", rel, url)).collect::<Vec<_>>().join(", ");

    link_header
}

fn generate_links(base_url: &str, page: u32, per_page: u32, total_count: u32) -> HashMap<&'static str, String> {
    let mut links = HashMap::new();

    if page > 1 {
        let prev_page_url = format!("{}/?page={}&per_page={}", base_url, page - 1, per_page);
        links.insert("prev", prev_page_url);
    }

    if page < (total_count as f32 / per_page as f32).ceil() as u32 { // per_page + 1
        let next_page_url = format!("{}/?page={}&per_page={}", base_url, page + 1, per_page);
        links.insert("next", next_page_url);
    }

    let first_page_url = format!("{}/?page=1&per_page={}", base_url, per_page);
    links.insert("first", first_page_url);

    let last_page_url = format!("{}/?page={}&per_page={}", base_url, (total_count as f32 / per_page as f32).ceil(), per_page);// per_page + 1,
    links.insert("last", last_page_url);
    links
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use itertools::Itertools;
    use rstest::rstest;
    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::common_utils::pagination::pagination_utils::{generate_api_link_header, generate_links};
    use maplit::hashmap;
    use spectral::assert_that;

    #[rstest]
    #[case("https://example.com/api", 1, 10, 100, hashmap ! {
    "first" => "https://example.com/api/?page=1&per_page=10".to_string(),
    "last" => "https://example.com/api/?page=10&per_page=10".to_string(),
    "next" => "https://example.com/api/?page=2&per_page=10".to_string(),
    })]
    #[case("https://example.com/api", 5, 10, 100, hashmap ! {
    "first" => "https://example.com/api/?page=1&per_page=10".to_string(),
    "last" => "https://example.com/api/?page=10&per_page=10".to_string(),
    "next" => "https://example.com/api/?page=6&per_page=10".to_string(),
    "prev" => "https://example.com/api/?page=4&per_page=10".to_string(),
    })]
    #[case("https://example.com/api", 10, 10, 100, hashmap ! {
    "first" => "https://example.com/api/?page=1&per_page=10".to_string(),
    "last" => "https://example.com/api/?page=10&per_page=10".to_string(),
    "prev" => "https://example.com/api/?page=9&per_page=10".to_string(),
    })]
    #[case("https://example.com/api", 1, 10, 5, hashmap ! {
    "first" => "https://example.com/api/?page=1&per_page=10".to_string(),
    "last" => "https://example.com/api/?page=1&per_page=10".to_string(),
    })]
// Test case where there is only one page of results
    #[case("https://example.com/api", 1, 10, 0, hashmap ! {})]
// Test case where total_count is 0
    #[case("https://example.com/api", 1, 10, 1, hashmap ! {
    "first" => "https://example.com/api/?page=1&per_page=10".to_string(),
    "last" => "https://example.com/api/?page=1&per_page=10".to_string(),
    })]
// Test case where there is exactly one item and one page
    #[case("https://example.com/api", 1, 10, 15, hashmap ! {
    "first" => "https://example.com/api/?page=1&per_page=10".to_string(),
    "last" => "https://example.com/api/?page=2&per_page=10".to_string(),
    "next" => "https://example.com/api/?page=2&per_page=10".to_string(),
    })]
// Test case where the last page is not a full page
    fn test_generate_links(
        #[case] base_url: &str,
        #[case] page: u32,
        #[case] per_page: u32,
        #[case] total_count: u32,
        #[case] expected_result: HashMap<&'static str, String>,
    ) {
        let actual_result = generate_links(base_url, page, per_page, total_count);
        actual_result.iter().sorted().zip(expected_result.iter().sorted()).inspect(|a| {
            println!("{:?}", a.0);
            println!("{:?}", a.1);
        })
            .for_each(|((act_rel, act_url), (exp_rel, exp_url))| {
                assert_that!(act_rel).is_equal_to(exp_rel);
                assert_that!(act_url).is_equal_to(exp_url);
            });
    }

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