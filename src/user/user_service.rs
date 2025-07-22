use std::sync::Arc;
use uuid::Uuid;
use crate::user::user_error::UserError;
use crate::user::user_model::{User, UserCreateRequest, UserResponse, UserUpdateRequest};
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
    
    pub async fn create_user(&self, request: UserCreateRequest) -> Result<(), UserError> {
        if self.user_repository.email_exists(&request.email).await? {
            return Err(UserError::EmailDuplicated)
        }

        if self.user_repository.username_exists(&request.username).await? {
            return Err(UserError::NameDuplicated)
        }
        
        let mut uuid = Uuid::new_v4();
        if self.user_repository.public_id_exists(&uuid.to_string()).await? {
            uuid = Uuid::new_v4();
        }
        
        let user = User {
            id: Default::default(),
            email: request.email,
            password: request.password,
            username: request.username,
            public_id: uuid.to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };

        let result = self.user_repository.create(user).await?;
        Ok(())
    }
    
    pub async fn update_user(&self, user_update: UserUpdateRequest) -> Result<(), UserError> {
        let result = self.user_repository.update(user_update.id, &user_update.email, &user_update.username).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::user_repository::MockUserRepository;
    use crate::user::user_model::{User};
    use mockall::predicate::*;
    use std::sync::Arc;
    use chrono::Utc;

    #[tokio::test]
    async fn create_user_success() {
        let mut user_repository = MockUserRepository::new();
        user_repository.expect_email_exists().returning(|_| Ok(false));
        user_repository.expect_username_exists().returning(|_| Ok(false));
        user_repository.expect_public_id_exists().returning(|_| Ok(false));

        let email = "test@example.com".to_string();
        let password = "test".to_string();
        let username = "test".to_string();
        let uuid = Uuid::new_v4().to_string();
        
        let user = User {
            id: Default::default(),
            email: email.clone(),
            password: password.clone(),
            username: username.clone(),
            public_id: uuid.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        
        let user_create_request = UserCreateRequest {
            email: email.clone(),
            password: password.clone(),
            username: username.clone(),
        };
        
        user_repository.expect_create()
            .withf(move |user_arg: &User| {
                user_arg.email == email && user_arg.password == password && user_arg.username == username
            })
            .times(1)
            .returning(|_| Ok(1));

        let user_service = UserService::new(Arc::new(user_repository));
  

        let result = user_service.create_user(user_create_request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn create_user_fails_on_duplicate_email() {
        let mut user_repository = MockUserRepository::new();

        let email = "test@example.com".to_string();
        let password = "test".to_string();
        let username = "test".to_string();
    
        user_repository.expect_email_exists()
            .with(eq(email.clone()))
            .times(1)
            .returning(|_| Ok(true));
        user_repository.expect_username_exists().returning(|_| Ok(false));
        user_repository.expect_create().never();

        let user_service = UserService::new(Arc::new(user_repository));
        let user_create = UserCreateRequest {
            email: email.clone(),
            password: password.clone(),
            username: username.clone(),
        };

        let result = user_service.create_user(user_create).await;
        assert!(matches!(result, Err(UserError::EmailDuplicated)));
    }
    
    #[tokio::test]
    async fn create_user_fails_on_duplicate_name() {
        let mut user_repository = MockUserRepository::new();
        
        let email = "test@example.com".to_string();
        let password = "test".to_string();
        let username = "test".to_string();
        
        user_repository.expect_email_exists().returning(|_| Ok(false));
        user_repository.expect_username_exists()
            .with(eq(username.clone()))
            .times(1)
            .returning(|_| Ok(true));
        user_repository.expect_create().never();
        
        let user_service = UserService::new(Arc::new(user_repository));
        let user_create = UserCreateRequest {
            email: email.clone(),
            password: password.clone(),
            username: username.clone(),
        };
        
        let result = user_service.create_user(user_create).await;
        assert!(matches!(result, Err(UserError::NameDuplicated)));
    }

    #[tokio::test]
    async fn get_user_success(){
        let mut user_repository = MockUserRepository::new();
        let user_id = 1;
        let name = "test";
        let password = "test";
        let email = "test@example.com";
        let expected_user = User {
            id: user_id,
            email: email.to_string(),
            password: password.to_string(),
            username: name.to_string(),
            public_id: Default::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        user_repository.expect_find_by_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Ok(expected_user.clone()));
        
        let user_service = UserService::new(Arc::new(user_repository));
        
        let result = user_service.get_user(user_id).await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.username, name);
        assert_eq!(user.email, email);
    }

    #[tokio::test]
    async fn get_user_fails_when_user_not_found() {
        let mut user_repository = MockUserRepository::new();
        let user_id = 9999;

        user_repository.expect_find_by_id()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Err(sqlx::Error::RowNotFound));

        let user_service = UserService::new(Arc::new(user_repository));

        let result = user_service.get_user(user_id).await;

        assert!(matches!(result, Err(UserError::NotFound)));
    }

    #[tokio::test]
    async fn get_user_fails_on_db_error() {
        let mut user_repository = MockUserRepository::new();
        let user_id = 1;

        user_repository.expect_find_by_id()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Err(sqlx::Error::PoolTimedOut));

        let user_service = UserService::new(Arc::new(user_repository));
        
        let result = user_service.get_user(user_id).await;
        
        assert!(matches!(result, Err(UserError::DatabaseError(_))));
    }
}