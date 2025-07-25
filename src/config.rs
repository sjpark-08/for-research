use std::env;

#[derive(Clone)]
pub struct Config {
    pub server_address: String,
    pub database_url: String,
    pub batch_google_api_key: String,
    pub google_api_key: String,
    pub jwt_secret: String,
    pub redis_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        
        Self {
            server_address: env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS must be set"),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            batch_google_api_key: env::var("BATCH_GOOGLE_API_KEY").expect("BATCH_GOOGLE_API_KEY must be set"),
            google_api_key: env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY must be set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            redis_url: env::var("REDIS_URL").expect("REDIS_URL must be set"),
        }
    }
}