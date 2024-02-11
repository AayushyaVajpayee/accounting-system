#[macro_export]
macro_rules! setup_routes {
    ($service_name:ident,$root:literal,$($path:literal,$http_mapping:expr),*) => {
        pub fn init_routes(config: &mut web::ServiceConfig, service: std::sync::Arc<dyn $service_name>) {
    let data = Data::new(service);
    config.service(map_endpoints_to_functions().app_data(data));
}

fn map_endpoints_to_functions() -> actix_web::Scope {
    actix_web::web::scope($root)
        $(
         .route($path, $http_mapping)

        )*
}

    };
}
#[cfg(test)]
#[macro_export]
macro_rules! get_and_create_api_test {
    ($entity_name:ident,$service_name:ident,$initialised_mock:expr,$get_uri:expr,$create_uri:literal,$create_request:expr,$expected:expr) => {
        use std::sync::Arc;
        use actix_web::test;
        let mocked = ($initialised_mock)();
        let mock: Arc<dyn $service_name> = Arc::new(mocked);
        let app_data = actix_web::web::Data::new(mock);
        let app = actix_web::App::new()
            .service(map_endpoints_to_functions())
            .app_data(app_data);
        let app_service = test::init_service(app).await;
        let request = test::TestRequest::get().uri(&$get_uri).to_request();
        let res: $entity_name = test::call_and_read_body_json(&app_service, request).await;
        assert_eq!(res, $expected);
        let request = test::TestRequest::post()
            .uri($create_uri)
            .set_json($create_request)
            .to_request();
        let _: uuid::Uuid = test::call_and_read_body_json(&app_service, request).await;
    };
}


#[cfg(test)]
#[macro_export]
macro_rules! get_and_create_api_test_v2 {
    ($entity_name:ident,$service_name:ident,$initialised_mock:expr,$get_uri:expr,$create_uri:literal,$create_request:expr,$expected:expr,$tenant_id:expr) => {
        use std::sync::Arc;
        use actix_web::test;
        use crate::accounting::user::user_models::SEED_USER_ID;
        let mocked = ($initialised_mock)();
        let mock: Arc<dyn $service_name> = Arc::new(mocked);
        let app_data = actix_web::web::Data::new(mock);
        let app = actix_web::App::new()
            .service(map_endpoints_to_functions())
            .app_data(app_data);
        let app_service = test::init_service(app).await;
        let request = test::TestRequest::get().uri(&$get_uri)
        .insert_header(("x-acc-tenant-id",$tenant_id.to_string()))
        .insert_header(("x-acc-user-id",SEED_USER_ID.to_string()))
        .to_request();
        let res: $entity_name = test::try_call_and_read_body_json(&app_service, request).await
        .unwrap();
        assert_eq!(res, $expected);
        let request = test::TestRequest::post()
            .uri($create_uri)
            .insert_header(("x-acc-tenant-id",$tenant_id.to_string()))
            .insert_header(("x-acc-user-id",SEED_USER_ID.to_string()))
            .set_json($create_request)
            .to_request();
        let _: uuid::Uuid = test::try_call_and_read_body_json(&app_service, request).await
        .unwrap();
    };
}