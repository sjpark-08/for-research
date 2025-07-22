use crate::app_state::AppState;
use crate::errors::{AppError, ErrorResponse};
use crate::user::user_model::{UserCreateRequest, UserResponse, UserUpdateRequest};
use actix_web::{get, post, put, web, HttpResponse};
use utoipa::OpenApi;

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
        (
            status = 200,
            body = UserResponse,
            description = "get user by id",
            content_type = "application/json"
        ),
        (
            status = 404,
            body = ErrorResponse,
            description = "user not found",
        )
    ),
    tags = ["User"]
)]
#[get("/{id}")]
pub async fn get_user(
    state: web::Data<AppState>,
    path: web::Path<i64>
) -> Result<HttpResponse, AppError> {
    let user_id: i64 = path.into_inner();
    let response = state.user_service.get_user(user_id).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "",
    responses(
        (
            status = 201,
            body = String,
            description = "user created",
        ),
    ),
    tags = ["User"]
)]
#[post("")]
pub async fn create_user(
    state: web::Data<AppState>,
    form: web::Json<UserCreateRequest>,
) -> Result<HttpResponse, AppError> {
    let response = state.user_service.create_user(form.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    put,
    path = "",
    responses(
        (
            status = 200,
            description = "update success"
        ),
    ),
    tags = ["User"]
)]
#[put("")]
pub async fn update_user(
    state: web::Data<AppState>,
    form: web::Json<UserUpdateRequest>,
) -> Result<HttpResponse, AppError> {
    let result = state.user_service.update_user(form.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}