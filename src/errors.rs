use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;
use crate::user::user_error::UserError;
use crate::youtube::youtube_data_api::youtube_data_api_error::YoutubeDataAPIError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    User(#[from] UserError),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    YoutubeDataAPI(#[from] YoutubeDataAPIError),
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    code: u16,
    message: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::User(e) => match e {
                UserError::NotFound => StatusCode::NOT_FOUND,
                UserError::EmailDuplicated => StatusCode::CONFLICT,
                UserError::NameDuplicated => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::YoutubeDataAPI(e) => match e {
                YoutubeDataAPIError::RequestError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                YoutubeDataAPIError::UploadPlayListNotFound => StatusCode::NOT_FOUND,
            } 
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
        };
        HttpResponse::build(status_code).json(error_response)
    }
}