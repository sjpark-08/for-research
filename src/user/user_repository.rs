use sqlx::{Error, MySqlPool, Result};
use crate::user::user_model::User;

#[derive(Clone)]
pub struct UserRepository {
    pub db_pool: MySqlPool,
}

impl UserRepository {
    pub fn new(db_pool: MySqlPool) -> Self {
        Self { db_pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<User, Error> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, name, email FROM users WHERE id = ?", 
            id
        )
            .fetch_one(&self.db_pool)
            .await?;
        
        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<User, Error> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, name, email FROM users WHERE email = ?",
            email
        )
            .fetch_one(&self.db_pool)
            .await?;

        Ok(user)
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool, Error> {
        let result = sqlx::query!("SELECT * FROM users WHERE email = ?", email)
            .fetch_optional(&self.db_pool)
            .await?;

        Ok(result.is_some())
    }

    pub async fn name_exists(&self, name: &str) -> Result<bool, Error> {
        let result = sqlx::query!("SELECT * FROM users WHERE name = ?", name)
            .fetch_optional(&self.db_pool)
            .await?;

        Ok(result.is_some())
    }

    pub async fn create(&self, name: &str, email: &str) -> Result<i64, Error> {
        let result = sqlx::query!(
            "INSERT INTO users (name, email) VALUES (?, ?)",
            name, email
        )
            .execute(&self.db_pool)
            .await?;

        Ok(result.last_insert_id() as i64)
    }
    
    pub async fn update(&self, id: i64, name: &str, email: &str) -> Result<(), Error> {
        sqlx::query!(
            "UPDATE users SET name = ?, email = ? WHERE id = ?",
            name, 
            email,
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
                id INT PRIMARY KEY AUTO_INCREMENT,
                name VARCHAR(255) NOT NULL UNIQUE,
                email VARCHAR(255) NOT NULL UNIQUE
            )
        ")
            .execute(pool)
            .await
            .unwrap();
    }
    #[sqlx::test]
    async fn create_user_success(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserRepository::new(pool);
        let name = "create_test";
        let email = "create_test@example.com";

        let new_user_id_result = user_repository.create(name, email).await;
        assert!(new_user_id_result.is_ok());
        let new_user_id = new_user_id_result.unwrap();

        let found_user = user_repository.find_by_id(new_user_id).await.unwrap();

        assert_eq!(found_user.id, new_user_id);
        assert_eq!(found_user.email, email);
        assert_eq!(found_user.name, name);
    }

    #[sqlx::test]
    async fn create_user_fails_on_duplicate_email(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserRepository::new(pool);
        let email = "duplicate@example.com";

        let first_creation_result = user_repository.create("user1", email).await;
        assert!(first_creation_result.is_ok());
        let second_creation_result = user_repository.create("user2", email).await;
        assert!(second_creation_result.is_err());
    }

    #[sqlx::test]
    async fn create_user_fails_on_duplicate_name(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserRepository::new(pool);
        let name = "duplicate";

        let first_creation_result = user_repository.create(name, "user1@example.com").await;
        assert!(first_creation_result.is_ok());
        let second_creation_result = user_repository.create(name, "user2@example.com").await;
        assert!(second_creation_result.is_err());
    }

    #[sqlx::test]
    async fn find_by_id_success(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserRepository::new(pool);
        let user_id = user_repository.create("test", "test").await.unwrap();

        let result = user_repository.find_by_id(user_id).await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.name, "test");
        assert_eq!(user.email, "test");
    }

    #[sqlx::test]
    async fn find_by_id_fails(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserRepository::new(pool);

        let result = user_repository.find_by_id(99).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound));
    }

    #[sqlx::test]
    async fn find_by_email_success(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserRepository::new(pool);
        let user_id = user_repository.create("test", "test").await.unwrap();

        let result = user_repository.find_by_email("test").await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.name, "test");
        assert_eq!(user.email, "test");
    }

    #[sqlx::test]
    async fn find_by_email_fails(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserRepository::new(pool);
        let user_id = user_repository.create("test", "test").await.unwrap();

        let result = user_repository.find_by_email("invalid@email").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound))
    }

    #[sqlx::test]
    async fn email_exists_success(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserRepository::new(pool);
        user_repository.create("test", "test@example.com").await.unwrap();

        let result = user_repository.email_exists("test@example.com").await;
        assert!(result.is_ok());
        let exists = result.unwrap();
        assert!(exists);
    }

    #[sqlx::test]
    async fn email_exists_fails(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserRepository::new(pool);

        let result = user_repository.email_exists("invalid@email").await;
        assert!(result.is_ok());
        let exists = result.unwrap();
        assert!(!exists);
    }

    #[sqlx::test]
    async fn name_exists_success(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserRepository::new(pool);
        user_repository.create("test", "test@example.com").await.unwrap();

        let result = user_repository.name_exists("test").await;
        assert!(result.is_ok());
        let exists = result.unwrap();
        assert!(exists);
    }

    #[sqlx::test]
    async fn name_exists_fails(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserRepository::new(pool);

        let result = user_repository.name_exists("invalid").await;
        assert!(result.is_ok());
        let exists = result.unwrap();
        assert!(!exists);
    }

    #[sqlx::test]
    async fn update_user_success(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserRepository::new(pool);
        let user_id = user_repository.create("test", "test@example.com").await.unwrap();

        let new_name = "updated";
        let new_email = "updated@example.com";
        let result = user_repository.update(user_id, new_name, new_email).await;
        assert!(result.is_ok());

        let user = user_repository.find_by_id(user_id).await.unwrap();
        assert_eq!(user.name, new_name);
        assert_eq!(user.email, new_email);
    }
    
    #[sqlx::test]
    async fn update_user_fails_on_duplicate_email(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserRepository::new(pool);
        let user1_id = user_repository.create("userA", "userA@email.com").await.unwrap();
        let user2_id = user_repository.create("userB", "userB@email.com").await.unwrap();
        
        let result = user_repository.update(user1_id, "userB", "userAA@email.com").await;
        assert!(result.is_err());
    }
    
    #[sqlx::test]
    async fn update_user_fails_on_duplicate_name(pool: MySqlPool) {
        init_schema(&pool).await;
        let user_repository = UserRepository::new(pool);
        let user1_id = user_repository.create("userA", "userA@email.com").await.unwrap();
        let user2_id = user_repository.create("userB", "userB@email.com").await.unwrap();
        
        let result = user_repository.update(user1_id, "userAA", "userB@email.com").await;
        assert!(result.is_err());
    }
}