use thiserror::Error;

#[derive(Error, Debug)]
pub enum YoutubeDataAPIError {
    #[error("HTTP 요청 실패")]
    RequestError(#[from] reqwest::Error),
}