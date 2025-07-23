use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;
use crate::auth::auth_model::AuthErrorResponse;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("토큰이 유효하지 않습니다.")]
    Unauthorized,
    
    #[error("존재하지 않는 사용자입니다.")]
    UserNotFound,
    
    #[error("데이터베이스 처리 오류: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("{0}")]
    InternalServerError(String),
    
    #[error("레디스 오류: {0}")]
    UnexpectedError(#[from] anyhow::Error),
    
    #[error("JWT 오류: {0}")]
    JWTError(#[from] jsonwebtoken::errors::Error),
    
    #[error("비밀번호가 일치하지 않습니다.")]
    InvalidPassword,
}

impl ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthError::Unauthorized => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = AuthErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
        };
        HttpResponse::build(status_code).json(error_response)
    }
}