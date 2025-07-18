use thiserror::Error;

#[derive(Error, Debug)]
pub enum YoutubeDataAPIError {
    #[error("HTTP 요청 실패")]
    RequestError(#[from] reqwest::Error),
    
    #[error("해당 채널의 업로드 목록을 찾을 수 없습니다.")]
    UploadPlayListNotFound,
}