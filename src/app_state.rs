use std::sync::Arc;
use sqlx::mysql::MySqlPoolOptions;
use crate::user::user_service::UserService;
use crate::config::Config;
use crate::gemini::gemini_api_util::GeminiAPIClient;
use crate::user::user_repository::UserSqlxRepository;
use crate::youtube::youtube_channel::youtube_channel_repository::YoutubeChannelSqlxRepository;
use crate::youtube::youtube_channel::youtube_channel_service::YoutubeChannelService;
use crate::youtube::youtube_data_api::youtube_data_api_util::YoutubeDataAPIClient;
use crate::youtube::youtube_video::youtube_raw_video_repository::YoutubeRawVideoSqlxRepository;
use crate::youtube::youtube_video::youtube_video_repository::YoutubeVideoSqlxRepository;
use crate::youtube::youtube_video::youtube_video_service::YoutubeVideoService;

#[derive(Clone)]
pub struct AppState {
    pub user_service: UserService,
    pub youtube_video_service: YoutubeVideoService,
    pub youtube_channel_service: YoutubeChannelService,
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
        
        let gemini_api_client = Arc::new(GeminiAPIClient::new(&config));
        
        let batch_youtube_data_client = YoutubeDataAPIClient::new(&config.batch_google_api_key);
        let youtube_data_client = YoutubeDataAPIClient::new(&config.google_api_key);
        let youtube_raw_video_repository = YoutubeRawVideoSqlxRepository::new(db_pool.clone());
        let youtube_video_repository = YoutubeVideoSqlxRepository::new(db_pool.clone());
        let youtube_video_service = YoutubeVideoService::new(
            Arc::new(batch_youtube_data_client),
            Arc::new(youtube_raw_video_repository),
            Arc::new(youtube_video_repository),
            Arc::clone(&gemini_api_client),
        );
        
        let youtube_channel_repository = YoutubeChannelSqlxRepository::new(db_pool.clone());
        let youtube_channel_service = YoutubeChannelService::new(
            Arc::new(youtube_channel_repository),
            Arc::new(youtube_data_client),
            Arc::clone(&gemini_api_client),
        );
        
        Self {
            user_service,
            youtube_video_service,
            youtube_channel_service,
        }
    }
}