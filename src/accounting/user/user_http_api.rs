use actix_web::{Responder, web};
use actix_web::web::Data;
use crate::accounting::user::user_models::CreateUserRequest;
use crate::accounting::user::user_service::{get_user_service, UserService};

async fn get_user_by_id(id: web::Path<i32>, data: web::Data<Box<dyn UserService + Send + Sync>>) -> actix_web::Result<impl Responder> {
    let p = data.get_user_by_id(&id).await;
    Ok(web::Json(p))
}

async fn create_user(request: web::Json<CreateUserRequest>,
                     data: Data<Box<dyn UserService + Send + Sync>>) -> actix_web::Result<impl Responder> {
    let p = data.create_user(&request.0).await;
    Ok(web::Json(p))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    let user_service = get_user_service();
    let data = Data::new(user_service);
    config.service(
        web::scope("user")
            .app_data(data)
            .route("/id/{id}", web::get().to(get_user_by_id))
            .route("/create", web::post().to(create_user))
    );
}