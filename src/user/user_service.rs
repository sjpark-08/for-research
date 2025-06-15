use actix_web::Error;
use crate::errors::AppError;
use crate::errors::AppError::Database;
use crate::user::user_error::UserError;
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
    
    pub async fn get_user(&self, user_id: i64) -> Result<UserResponse, UserError> {
        self.user_repository
            .find_by_id(user_id)
            .await
            .map_err(|db_error| {
                match db_error {
                    sqlx::Error::RowNotFound => UserError::NotFound,
                    _ => db_error.into(),
                }
            })
            .map(|user| user.into())
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