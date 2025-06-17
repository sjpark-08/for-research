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

    pub async fn create(&self, name: &str, email: &str) -> Result<(), Error> {
        sqlx::query!(
            "INSERT INTO users (name, email) VALUES (?, ?)",
            name, email
        )
            .execute(&self.db_pool)
            .await?;

        Ok(())
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