use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("사용자를 찾을 수 없습니다.")]
    NotFound,
    
    #[error("이미 존재하는 이메일입니다.")]
    EmailDuplicated,
    
    #[error("이미 존재하는 닉네임입니다.")]
    NameDuplicated,
    
    #[error("데이터베이스 처리 오류: {0}")]
    DatabaseError(#[from] sqlx::Error),
}