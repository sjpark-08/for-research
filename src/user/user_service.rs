use std::sync::Arc;
use crate::user::user_error::UserError;
use crate::user::user_model::{UserCreate, UserResponse, UserUpdate};
use crate::user::user_repository::UserRepository;

#[derive(Clone)]
pub struct UserService {
    pub user_repository: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
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
    
    pub async fn create_user(&self, user_create: UserCreate) -> Result<(), UserError> {
        if self.user_repository.email_exists(&user_create.email).await? {
            return Err(UserError::EmailDuplicated)
        }

        if self.user_repository.name_exists(&user_create.name).await? {
            return Err(UserError::NameDuplicated)
        }

        let result = self.user_repository.create(&user_create.name, &user_create.email).await?;
        Ok(())
    }
    
    pub async fn update_user(&self, user_update: UserUpdate) -> Result<(), UserError> {
        let result = self.user_repository.update(user_update.id, &user_update.name, &user_update.email).await?;
        Ok(())
    }
}