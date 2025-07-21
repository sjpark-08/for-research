use thiserror::Error;

#[derive(Error, Debug)]
pub enum YoutubeChannelError {
    #[error("채널 '{0}'을 찾을 수 없습니다.")]
    ChannelNotFound(String),
    
    #[error("채널 '{0}'의 데이터가 이미 존재합니다.")]
    ChannelDuplicated(String),
}