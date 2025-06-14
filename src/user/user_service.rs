use actix_web::Error;
use crate::user::user_model::{User, UserCreate, UserResponse};
use crate::user::user_repository::UserRepository;

#[derive(Clone)]
pub struct UserService {
    pub user_repository: UserRepository,
}

impl UserService {
    pub fn new(user_repository: UserRepository) -> Self {
        Self { user_repository }
    }
    
    pub async fn get_user(&self, user_id: i64) -> Result<UserResponse, Error> {
        let user = self.user_repository.find_by_id(user_id).await.unwrap();
        Ok(user.into())
    }
    
    pub async fn create_user(&self, user_create: UserCreate) -> Result<u64, Error> {
        let result = self.user_repository.create(&user_create.name, &user_create.email).await.unwrap();
        Ok(result)
    }
}