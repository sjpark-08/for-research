use std::env;

#[derive(Clone)]
pub struct Config {
    pub server_address: String,
    pub database_url: String,
    pub test_database_url: String
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        
        Self {
            server_address: env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS must be set"),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            test_database_url: env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set")
        }
    }
}