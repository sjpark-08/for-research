use r2d2::Pool;
use redis::{Client, Commands};

pub struct RedisRepository {
    pool: Pool<Client>,
}

impl RedisRepository {
    pub fn new(pool: Pool<Client>) -> Self {
        Self { pool }
    }
    
    pub async fn set_refresh_token(&self, user_public_id: &str, refresh_token: &str) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get()?;
        let key = format!("user:{}:refresh_token", user_public_id);
        let value = refresh_token;
        let _: () = conn.set_ex(&key, &value, 86400)?;
        
        Ok(())
    }
    
    pub async fn get_refresh_token(&self, user_public_id: &str) -> Result<(Option<String>), anyhow::Error> {
        let mut conn = self.pool.get()?;
        let key = format!("user:{}:refresh_token", user_public_id);
        let value: Option<String> = conn.get(&key)?;
        
        Ok(value)
    }
    
    pub async fn delete_refresh_token(&self, user_public_id: &str) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get()?;
        let key = format!("user:{}:refresh_token", user_public_id);
        let _: () = conn.del(&key)?;
        
        Ok(())
    }
}