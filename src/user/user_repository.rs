use async_trait::async_trait;
use mockall::automock;
use sqlx::{Error, MySqlPool, Result};
use crate::user::user_model::User;

#[automock]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: i64) -> Result<User, Error>;
    async fn find_by_email(&self, email: &str) -> Result<User, Error>;
    async fn email_exists(&self, email: &str) -> Result<bool, Error>;
    async fn username_exists(&self, username: &str) -> Result<bool, Error>;
    async fn create(&self, email: &str, password: &str, username: &str) -> Result<i64, Error>;
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
                SELECT id, email, password, username, created_at, updated_at
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
                SELECT id, email, password, username, created_at, updated_at
                FROM users
                WHERE email = ?
            "#,
            email
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

    async fn create(&self, email: &str, password: &str, username: &str) -> Result<i64, Error> {
        let result = sqlx::query!(
            r#"
                INSERT INTO users (email, password, username)
                VALUES (?, ?, ?)
            "#,
            email, password, username
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
    use super::*;

    async fn init_schema(pool: &MySqlPool) {
        sqlx::query("
            CREATE TABLE IF NOT EXISTS users (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                email VARCHAR(255) NOT NULL UNIQUE,
                password VARCHAR(255) NOT NULL,
                username VARCHAR(255) NOT NULL UNIQUE,
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

        let new_user_id_result = user_repository.create(email, password, username).await;
        assert!(new_user_id_result.is_ok());
        let new_user_id = new_user_id_result.unwrap();

        let found_user = user_repository.find_by_id(new_user_id).await.unwrap();

        assert_eq!(found_user.id, new_user_id);
        assert_eq!(found_user.email, email);
        assert_eq!(found_user.password, password);
        assert_eq!(found_user.username, username);
    }

    #[sqlx::test]
    async fn create_user_fails_on_duplicate_email(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let email = "duplicate@example.com";

        let first_creation_result = user_repository.create(email, "password", "user1").await;
        assert!(first_creation_result.is_ok());
        let second_creation_result = user_repository.create(email, "password", "user2").await;
        assert!(second_creation_result.is_err());
    }

    #[sqlx::test]
    async fn create_user_fails_on_duplicate_name(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let username = "duplicate";

        let first_creation_result = user_repository.create("user1@example.com", "password", username).await;
        assert!(first_creation_result.is_ok());
        let second_creation_result = user_repository.create("user2@example.com", "password", username).await;
        assert!(second_creation_result.is_err());
    }

    #[sqlx::test]
    async fn find_by_id_success(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let user_id = user_repository.create("test", "test", "test").await.unwrap();

        let result = user_repository.find_by_id(user_id).await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.username, "test");
        assert_eq!(user.email, "test");
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
        let user_id = user_repository.create("test", "test", "test").await.unwrap();

        let result = user_repository.find_by_email("test").await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.username, "test");
        assert_eq!(user.email, "test");
    }

    #[sqlx::test]
    async fn find_by_email_fails(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let user_id = user_repository.create("test", "test", "test").await.unwrap();

        let result = user_repository.find_by_email("invalid@email").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound))
    }

    #[sqlx::test]
    async fn email_exists_success(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        user_repository.create("test@example.com", "password", "test").await.unwrap();

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
        user_repository.create("test@example.com", "password", "test").await.unwrap();

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
        let user_id = user_repository.create("test@example.com", "password", "test").await.unwrap();

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
        let user1_id = user_repository.create("userA@email.com", "password", "userA").await.unwrap();
        let user2_id = user_repository.create("userB@email.com", "password", "userB",).await.unwrap();

        let result = user_repository.update(user1_id,"userAA@email.com", "userB").await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn update_user_fails_on_duplicate_name(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserSqlxRepository::new(pool);
        let user1_id = user_repository.create("userA@email.com", "password", "userA").await.unwrap();
        let user2_id = user_repository.create("userB@email.com", "password", "userB").await.unwrap();

        let result = user_repository.update(user1_id,"userB@email.com","userAA").await;
        assert!(result.is_err());
    }
}