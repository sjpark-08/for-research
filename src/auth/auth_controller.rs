use actix_web::{post, web, HttpRequest, HttpResponse};
use utoipa::OpenApi;
use crate::app_state::AppState;
use crate::auth::auth_model::{AuthenticatedUser, LoginRequest};
use crate::errors::AppError;

#[derive(OpenApi)]
#[openapi(
    paths(
        login,
        logout,
        refresh_token,
    ),
    components(),
    tags(
        (name = "Auth", description = "Auth management endpoints"),
    )
)]
pub struct AuthApi;

pub fn auth_api(config: &mut web::ServiceConfig) {
    config.service(login)
        .service(logout)
        .service(refresh_token);
}

#[utoipa::path(
    post,
    path = "/login",
    responses(
        (
            status = 200,
            description = "Login successful",
            content_type = "application/json",
        )
    ),
    tags = ["Auth"]
)]
#[post("/login")]
pub async fn login(
    state: web::Data<AppState>,
    form: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let response = state.auth_service.login(form.into_inner()).await?;
    Ok(response)
}

#[utoipa::path(
    post,
    path = "/logout",
    responses(
        (
            status = 200,
            description = "Logout successful",
            content_type = "application/json",
        )
    ),
    tags = ["Auth"]
)]
#[post("/logout")]
pub async fn logout(
    state: web::Data<AppState>,
    request: HttpRequest,
    auth_user: AuthenticatedUser
) -> Result<HttpResponse, AppError> {
    let response = state.auth_service.logout(request).await?;
    Ok(response)
}

#[utoipa::path(
    post,
    path = "/refresh",
    responses(
        (
            status = 200,
            description = "Refresh successful",
            content_type = "application/json",
        )
    ),
    tags = ["Auth"]
)]
#[post("/refresh")]
pub async fn refresh_token(
    state: web::Data<AppState>,
    request: HttpRequest
) -> Result<HttpResponse, AppError> {
    let response = state.auth_service.refresh_token(request).await?;
    Ok(response)
}