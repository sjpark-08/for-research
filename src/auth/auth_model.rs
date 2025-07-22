use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug)]
pub struct AuthenticatedUser {
    pub public_id: String,
}

#[derive(Serialize)]
pub struct AuthErrorResponse {
    pub code: u16,
    pub message: String,
}
