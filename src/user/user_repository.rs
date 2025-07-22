use async_trait::async_trait;
use mockall::automock;
use sqlx::{Error, MySqlPool, Result};
use crate::user::user_model::User;

#[automock]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: i64) -> Result<User, Error>;
    
    async fn find_by_email(&self, email: &str) -> Result<User, Error>;
    
    async fn find_by_public_id(&self, public_id: &str) -> Result<User, Error>;
    
    async fn email_exists(&self, email: &str) -> Result<bool, Error>;
    
    async fn username_exists(&self, username: &str) -> Result<bool, Error>;
    
    async fn public_id_exists(&self, public_id: &str) -> Result<bool, Error>;
    
    async fn create(&self, user: User) -> Result<i64, Error>;
    
    async fn update(&self, id: i64, email: &str, username: &str) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct UserSqlxRepository {
    pub db_pool: MySqlPool,
}

impl UserSqlxRepository {
    pub fn new(db_pool: MySqlPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl UserRepository for UserSqlxRepository {
    async fn find_by_id(&self, id: i64) -> Result<User, Error> {
        let user = sqlx::query_as!(
            User,
            r#"
                SELECT id, email, password, username, public_id, created_at, updated_at
                FROM users
                WHERE id = ?
            "#,
            id as i64
        )
            .fetch_one(&self.db_pool)
            .await?;
        
        Ok(user)
    }
    async fn find_by_email(&self, email: &str) -> Result<User, Error> {
        let user = sqlx::query_as!(
            User,
            r#"
                SELECT id, email, password, username, public_id, created_at, updated_at
                FROM users
                WHERE email = ?
            "#,
            email
        )
            .fetch_one(&self.db_pool)
            .await?;

        Ok(user)
    }
    
    async fn find_by_public_id(&self, public_id: &str) -> Result<User, Error> {
        let user = sqlx::query_as!(
            User,
            r#"
                SELECT id, email, password, username, public_id, created_at, updated_at
                FROM users
                WHERE public_id = ?
            "#,
            public_id
        )
            .fetch_one(&self.db_pool)
            .await?;
        
        Ok(user)
    }

    async fn email_exists(&self, email: &str) -> Result<bool, Error> {
        let result = sqlx::query!(
            r#"
                SELECT *
                FROM users
                WHERE email = ?
            "#,
            email
        )
            .fetch_optional(&self.db_pool)
            .await?;

        Ok(result.is_some())
    }

    async fn username_exists(&self, name: &str) -> Result<bool, Error> {
        let result = sqlx::query!(
            r#"
                SELECT *
                FROM users
                WHERE username = ?
            "#,
            name
        )
            .fetch_optional(&self.db_pool)
            .await?;

        Ok(result.is_some())
    }
    
    async fn public_id_exists(&self, public_id: &str) -> Result<bool, Error> {
        let result = sqlx::query!(
            r#"
                SELECT *
                FROM users
                WHERE public_id = ?
            "#,
            public_id
        )
            .fetch_optional(&self.db_pool)
            .await?;
        
        Ok(result.is_some())
    }

    async fn create(&self, user: User) -> Result<i64, Error> {
        let result = sqlx::query!(
            r#"
                INSERT INTO users (email, password, username, public_id)
                VALUES (?, ?, ?, ?)
            "#,
            user.email, user.password, user.username, user.public_id
        )
            .execute(&self.db_pool)
            .await?;

        Ok(result.last_insert_id() as i64)
    }
    
    async fn update(&self, id: i64, email: &str, username: &str) -> Result<(), Error> {
        sqlx::query!(
            r#"
                UPDATE users
                SET email = ?, username = ?
                WHERE id = ?
            "#,
            email,
            username,
            id
        )
            .execute(&self.db_pool)
            .await?;
            
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use super::*;

    async fn init_schema(pool: &MySqlPool) {
        sqlx::query("
            CREATE TABLE IF NOT EXISTS users (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                email VARCHAR(255) NOT NULL UNIQUE,
                password VARCHAR(255) NOT NULL,
                username VARCHAR(255) NOT NULL UNIQUE,
                public_id VARCHAR(255) NOT NULL UNIQUE,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
            )
        ")
            .execute(pool)
            .await
            .unwrap();
    }
    #[sqlx::test]
    async fn create_user_success(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let email = "create_test@example.com";
        let password = "test_password";
        let username = "create_test";
        let uuid = Uuid::new_v4().to_string();
        let user = User {
            id: Default::default(),
            email: email.to_string(),
            password: password.to_string(),
            username: username.to_string(),
            public_id: uuid.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };

        let new_user_id_result = user_repository.create(user).await;
        assert!(new_user_id_result.is_ok());
        let new_user_id = new_user_id_result.unwrap();

        let found_user = user_repository.find_by_id(new_user_id).await.unwrap();

        assert_eq!(found_user.id, new_user_id);
        assert_eq!(found_user.email, email);
        assert_eq!(found_user.password, password);
        assert_eq!(found_user.username, username);
        assert_eq!(found_user.public_id, uuid);
    }

    #[sqlx::test]
    async fn create_user_fails_on_duplicate_email(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let email = "duplicate@example.com";
        let password = "test_password";
        let username1 = "user1";
        let username2 = "user2";
        let uuid1 = Uuid::new_v4().to_string();
        let uuid2 = Uuid::new_v4().to_string();
        let user1 = User {
            id: Default::default(),
            email: email.to_string(),
            password: password.to_string(),
            username: username1.to_string(),
            public_id: uuid1.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        let user2 = User {
            id: Default::default(),
            email: email.to_string(),
            password: password.to_string(),
            username: username2.to_string(),
            public_id: uuid2.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };

        let first_creation_result = user_repository.create(user1).await;
        assert!(first_creation_result.is_ok());
        let second_creation_result = user_repository.create(user2).await;
        assert!(second_creation_result.is_err());
    }

    #[sqlx::test]
    async fn create_user_fails_on_duplicate_username(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let username = "duplicate";
        let email1 = "user1@example.com";
        let email2 = "user2@example.com";
        let password = "test_password";
        let uuid1 = Uuid::new_v4().to_string();
        let uuid2 = Uuid::new_v4().to_string();
        
        let user1 = User {
            id: Default::default(),
            email: email1.to_string(),
            password: password.to_string(),
            username: username.to_string(),
            public_id: uuid1.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        let user2 = User {
            id: Default::default(),
            email: email1.to_string(),
            password: password.to_string(),
            username: username.to_string(),
            public_id: uuid2.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        
        let first_creation_result = user_repository.create(user1).await;
        assert!(first_creation_result.is_ok());
        let second_creation_result = user_repository.create(user2).await;
        assert!(second_creation_result.is_err());
    }
        
        #[sqlx::test]
        async fn create_user_fails_on_duplicate_public_id(pool: MySqlPool) {
            init_schema(&pool).await;
            let user_repository = UserSqlxRepository::new(pool);
            let username1 = "user1";
            let username2 = "user2";
            let email1 = "user1@example.com";
            let email2 = "user2@example.com";
            let password = "test_password";
            let uuid = Uuid::new_v4().to_string();
            
            let user1 = User {
                id: Default::default(),
                email: email1.to_string(),
                password: password.to_string(),
                username: username1.to_string(),
                public_id: uuid.clone(),
                created_at: Default::default(),
                updated_at: Default::default(),
            };
            let user2 = User {
                id: Default::default(),
                email: email2.to_string(),
                password: password.to_string(),
                username: username2.to_string(),
                public_id: uuid.clone(),
                created_at: Default::default(),
                updated_at: Default::default(),
            };

        let first_creation_result = user_repository.create(user1).await;
        assert!(first_creation_result.is_ok());
        let second_creation_result = user_repository.create(user2).await;
        assert!(second_creation_result.is_err());
    }

    #[sqlx::test]
    async fn find_by_id_success(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let uuid = Uuid::new_v4().to_string();
        let user = User {
            id: Default::default(),
            email: "test".to_string(),
            password: "test".to_string(),
            username: "test".to_string(),
            public_id: uuid.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        let user_id = user_repository.create(user).await.unwrap();

        let result = user_repository.find_by_id(user_id).await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.username, "test");
        assert_eq!(user.email, "test");
        assert_eq!(user.password, "test");
        assert_eq!(user.public_id, uuid);
    }

    #[sqlx::test]
    async fn find_by_id_fails(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);

        let result = user_repository.find_by_id(99).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound));
    }

    #[sqlx::test]
    async fn find_by_email_success(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let uuid = Uuid::new_v4().to_string();
        let user = User {
            id: Default::default(),
            email: "test".to_string(),
            password: "test".to_string(),
            username: "test".to_string(),
            public_id: uuid.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        let user_id = user_repository.create(user).await.unwrap();

        let result = user_repository.find_by_email("test").await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.username, "test");
        assert_eq!(user.email, "test");
        assert_eq!(user.password, "test");
        assert_eq!(user.public_id, uuid);
    }

    #[sqlx::test]
    async fn find_by_email_fails(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let uuid = Uuid::new_v4().to_string();
        let user = User {
            id: Default::default(),
            email: "test".to_string(),
            password: "test".to_string(),
            username: "test".to_string(),
            public_id: uuid.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        let user_id = user_repository.create(user).await.unwrap();

        let result = user_repository.find_by_email("invalid@email").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound))
    }

    #[sqlx::test]
    async fn email_exists_success(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let uuid = Uuid::new_v4().to_string();
        let user = User {
            id: Default::default(),
            email: "test@example.com".to_string(),
            password: "test".to_string(),
            username: "test".to_string(),
            public_id: uuid.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        user_repository.create(user).await.unwrap();

        let result = user_repository.email_exists("test@example.com").await;
        assert!(result.is_ok());
        let exists = result.unwrap();
        assert!(exists);
    }

    #[sqlx::test]
    async fn email_exists_fails(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);

        let result = user_repository.email_exists("invalid@email").await;
        assert!(result.is_ok());
        let exists = result.unwrap();
        assert!(!exists);
    }

    #[sqlx::test]
    async fn name_exists_success(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let uuid = Uuid::new_v4().to_string();
        let user = User {
            id: Default::default(),
            email: "test@example.com".to_string(),
            password: "test".to_string(),
            username: "test".to_string(),
            public_id: uuid.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        user_repository.create(user).await.unwrap();

        let result = user_repository.username_exists("test").await;
        assert!(result.is_ok());
        let exists = result.unwrap();
        assert!(exists);
    }

    #[sqlx::test]
    async fn name_exists_fails(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);

        let result = user_repository.username_exists("invalid").await;
        assert!(result.is_ok());
        let exists = result.unwrap();
        assert!(!exists);
    }

    #[sqlx::test]
    async fn update_user_success(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let uuid = Uuid::new_v4().to_string();
        let user = User {
            id: Default::default(),
            email: "test@example.com".to_string(),
            password: "test".to_string(),
            username: "test".to_string(),
            public_id: uuid.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        let user_id = user_repository.create(user).await.unwrap();

        let new_username = "updated";
        let new_email = "updated@example.com";
        let result = user_repository.update(user_id, new_email, new_username).await;
        assert!(result.is_ok());

        let user = user_repository.find_by_id(user_id).await.unwrap();
        assert_eq!(user.username, new_username);
        assert_eq!(user.email, new_email);
    }

    #[sqlx::test]
    async fn update_user_fails_on_duplicate_email(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let uuid1 = Uuid::new_v4().to_string();
        let uuid2 = Uuid::new_v4().to_string();
        let user1 = User {
            id: Default::default(),
            email: "userA@email.com".to_string(),
            password: "password".to_string(),
            username: "userA".to_string(),
            public_id: uuid1.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        let user2 = User {
            id: Default::default(),
            email: "userB@email.com".to_string(),
            password: "password".to_string(),
            username: "userB".to_string(),
            public_id: uuid2.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        let user1_id = user_repository.create(user1).await.unwrap();
        let user2_id = user_repository.create(user2).await.unwrap();

        let result = user_repository.update(user1_id,"userAA@email.com", "userB").await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn update_user_fails_on_duplicate_name(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let uuid1 = Uuid::new_v4().to_string();
        let uuid2 = Uuid::new_v4().to_string();
        let user1 = User {
            id: Default::default(),
            email: "userA@email.com".to_string(),
            password: "password".to_string(),
            username: "userA".to_string(),
            public_id: uuid1.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        let user2 = User {
            id: Default::default(),
            email: "userB@email.com".to_string(),
            password: "password".to_string(),
            username: "userB".to_string(),
            public_id: uuid2.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };
        let user1_id = user_repository.create(user1).await.unwrap();
        let user2_id = user_repository.create(user2).await.unwrap();

        let result = user_repository.update(user1_id,"userB@email.com","userAA").await;
        assert!(result.is_err());
    }
}