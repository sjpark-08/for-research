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

    pub async fn create(&self, name: &str, email: &str) -> Result<u64> {
        let result = sqlx::query!(
            "INSERT INTO users (name, email) VALUES (?, ?)",
            name, email
        )
            .execute(&self.db_pool)
            .await?;

        Ok(result.last_insert_id())
    }
}