use std::sync::Arc;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use crate::user::user_service::UserService;
use crate::config::Config;
use crate::user::user_repository::{UserRepository, UserSqlxRepository};

#[derive(Clone)]
pub struct AppState {
    pub user_service: UserService,
}

impl AppState {
    pub async fn new(config: &Config) -> Self {
        let db_pool = MySqlPoolOptions::new()
            .max_connections(10)
            .connect(&config.database_url)
            .await
            .expect("Failed to connect to database");
        
        let user_repository = UserSqlxRepository::new(db_pool.clone());
        
        let user_service = UserService::new(Arc::new(user_repository));
        
        Self {
            user_service,
        }
    }
}