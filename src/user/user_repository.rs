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
        let name = "test";
        let email = "test@example.com";

        let new_user_id_result = user_repository.create(name, email).await;
        if let Err(e) = &new_user_id_result {
            eprintln!("DB create error: {:?}", e);
        }
        assert!(new_user_id_result.is_ok());
        let new_user_id = new_user_id_result.unwrap();

        let found_user_result = user_repository.find_by_id(new_user_id).await;
        assert!(found_user_result.is_ok());
        let found_user = found_user_result.unwrap();

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
}