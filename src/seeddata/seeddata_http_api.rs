use std::sync::Arc;
use actix_web::{Responder, Scope, web};

use crate::seeddata::seed_service::{get_seed_service, SeedService};

pub async fn init_schema_and_create_seed_data(data: web::Data<Arc<dyn SeedService>>) -> actix_web::Result<impl Responder> {
    data.copy_tables().await;
    Ok("oh yeah seed data".to_string())
}

pub fn init_routes(config: &mut web::ServiceConfig, seed_service: Arc<dyn SeedService>) {
    let data = web::Data::new(seed_service);
    config.service(map_endpoints_to_functions()
        .app_data(data));
}

fn map_endpoints_to_functions() -> Scope {
    web::scope("seeddata")
        .route("/initdb", web::post().to(init_schema_and_create_seed_data))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use actix_web::{App, test};
    use async_trait::async_trait;
    use spectral::assert_that;

    use crate::seeddata::seed_service::SeedService;
    use crate::seeddata::seeddata_http_api::map_endpoints_to_functions;

    struct MockSeedService {}

    #[async_trait]
    impl SeedService for MockSeedService {
        async fn copy_tables(&self) {}
    }

    #[tokio::test]
    async fn test_init_schema_and_create_seed_data() {
        let mock: Arc<dyn SeedService> =
            Arc::new(MockSeedService {});
        let app_data = actix_web::web::Data::new(mock);
        let app = App::new()
            .service(map_endpoints_to_functions())
            .app_data(app_data);
        let app_service = test::init_service(app).await;
        let request = test::TestRequest::post()
            .uri("/seeddata/initdb")
            .to_request();
        let res = test::call_service(&app_service, request).await;
        assert_that!(res.status()).matches(|a|a.is_success())
    }
}