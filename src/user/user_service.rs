use actix_web::Error;
use crate::user::user_model::{User, UserCreate, UserResponse, UserUpdate};
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
    
    pub async fn create_user(&self, user_create: UserCreate) -> Result<(), Error> {
        let result = self.user_repository.create(&user_create.name, &user_create.email).await.unwrap();
        Ok(())
    }
    
    pub async fn update_user(&self, user_update: UserUpdate) -> Result<(), Error> {
        let result = self.user_repository.update(user_update.id, &user_update.name, &user_update.email).await.unwrap();
        Ok(())
    }
}