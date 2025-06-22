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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::user_repository::MockUserRepository;
    use crate::user::user_model::UserCreate;
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn create_user_success() {
        let mut user_repository = MockUserRepository::new();
        user_repository.expect_email_exists().returning(|_| Ok(false));
        user_repository.expect_name_exists().returning(|_| Ok(false));

        let name = "test".to_string();
        let email = "test@example.com".to_string();

        user_repository.expect_create()
            .with(eq(name.clone()), eq(email.clone()))
            .times(1)
            .returning(|_, _| Ok(1));

        let user_service = UserService::new(Arc::new(user_repository));
        let user_create = UserCreate {
            name: name.clone(),
            email: email.clone(),
        };

        let result = user_service.create_user(user_create).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn create_user_fails_on_duplicate_email() {
        let mut user_repository = MockUserRepository::new();

        let name = "test".to_string();
        let email = "test@example.com".to_string();
    
        user_repository.expect_email_exists()
            .with(eq(email.clone()))
            .times(1)
            .returning(|_| Ok(true));
        user_repository.expect_name_exists().returning(|_| Ok(false));
        user_repository.expect_create().never();

        let user_service = UserService::new(Arc::new(user_repository));
        let user_create = UserCreate {
            name: name.clone(),
            email: email.clone(),
        };

        let result = user_service.create_user(user_create).await;
        assert!(matches!(result, Err(UserError::EmailDuplicated)));
    }
    
    #[tokio::test]
    async fn create_user_fails_on_duplicate_name() {
        let mut user_repository = MockUserRepository::new();
        
        let name = "test".to_string();
        let email = "test@example.com".to_string();
        
        user_repository.expect_email_exists().returning(|_| Ok(false));
        user_repository.expect_name_exists()
            .with(eq(name.clone()))
            .times(1)
            .returning(|_| Ok(true));
        user_repository.expect_create().never();
        
        let user_service = UserService::new(Arc::new(user_repository));
        let user_create = UserCreate {
            name: name.clone(),
            email: email.clone(),
        };
        
        let result = user_service.create_user(user_create).await;
        assert!(matches!(result, Err(UserError::NameDuplicated)));
    }
}