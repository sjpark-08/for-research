use crate::user::user_model;
use actix_web::{get, post, put, web, HttpResponse, Responder, Scope};
use utoipa::OpenApi;
use crate::app_state::AppState;
use crate::user::user_model::{UserCreate, UserUpdate};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_user,
        create_user,
        update_user,
    ),
    components(),
    tags(
        (name = "User", description = "User management endpoints")
    )
)]
pub struct UserApi;

pub fn user_api(config: &mut web::ServiceConfig) {
    config.service(get_user)
        .service(create_user)
        .service(update_user);
}

#[utoipa::path(
    get,
    path = "/{id}",
    responses(
        (status = 200, description = "Hello message", body = String),
    ),
    tags = ["User"]
)]
#[get("/{id}")]
pub async fn get_user(
    state: web::Data<AppState>,
    path: web::Path<i64>
) -> impl Responder {
    let user_id: i64 = path.into_inner();
    
    match state.user_service.get_user(user_id).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::NotFound().body("User not found"),
    }
}

#[utoipa::path(
    post,
    path = "",
    responses(
        (status = 201, description = "user created", body = String),
    ),
    tags = ["User"]
)]
#[post("")]
pub async fn create_user(
    state: web::Data<AppState>,
    form: web::Json<UserCreate>,
) -> impl Responder {
    match state.user_service.create_user(form.into_inner()).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(_) => HttpResponse::InternalServerError().body("Error creating user"),
    }
}

#[utoipa::path(
    put,
    path = "",
    responses(
        (status = 200, description = "update success", body = String),
    ),
    tags = ["User"]
)]
#[put("")]
pub async fn update_user(
    state: web::Data<AppState>,
    form: web::Json<UserUpdate>,
) -> impl Responder {
    match state.user_service.update_user(form.into_inner()).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(_) => HttpResponse::InternalServerError().body("Error updating user"),
    }
}