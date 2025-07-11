use std::sync::Arc;
use sqlx::mysql::MySqlPoolOptions;
use crate::user::user_service::UserService;
use crate::config::Config;
use crate::user::user_repository::UserSqlxRepository;
use crate::youtube::youtube_data_api::youtube_data_api_util::YoutubeDataAPIClient;
use crate::youtube::youtube_video::youtube_raw_video_repository::YoutubeRawVideoSqlxRepository;
use crate::youtube::youtube_video::youtube_video_service::YoutubeVideoService;

#[derive(Clone)]
pub struct AppState {
    pub user_service: UserService,
    pub youtube_video_service: YoutubeVideoService,
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
        
        let youtube_data_client = YoutubeDataAPIClient::new(&config);
        let youtube_raw_video_repository = YoutubeRawVideoSqlxRepository::new(db_pool.clone());
        let youtube_video_service = YoutubeVideoService::new(
            Arc::new(youtube_data_client),
            Arc::new(youtube_raw_video_repository)
        );
        
        Self {
            user_service,
            youtube_video_service
        }
    }
}