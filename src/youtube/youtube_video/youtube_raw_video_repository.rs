use async_trait::async_trait;
use mockall::automock;
use sqlx::{Error, MySqlPool};
use crate::youtube::youtube_video::youtube_video_model::YoutubeRawVideo;

#[automock]
#[async_trait]
pub trait YoutubeRawVideoRepository:  Send + Sync {
    async fn find_by_video_id(&self, video_id: &str) -> Result<YoutubeRawVideo, Error>;
    
    async fn save(&self, raw_video: &YoutubeRawVideo) -> Result<i64, Error>;
    
    async fn save_many(&self, raw_videos: &[YoutubeRawVideo]) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct YoutubeRawVideoSqlxRepository {
    pub db_pool: MySqlPool,
}

impl YoutubeRawVideoSqlxRepository {
    pub fn new(db_pool: MySqlPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl YoutubeRawVideoRepository for YoutubeRawVideoSqlxRepository {
    async fn find_by_video_id(&self, video_id: &str) -> Result<YoutubeRawVideo, Error> {
        let raw_video = sqlx::query_as!(
            YoutubeRawVideo,
            r#"
                SELECT id, video_id, raw_metadata, created_at, updated_at 
                FROM youtube_raw_videos 
                WHERE video_id = ?
            "#,
            video_id
        )
            .fetch_one(&self.db_pool)
            .await?;
        Ok(raw_video)
    }

    async fn save(&self, raw_video: &YoutubeRawVideo) -> Result<i64, Error> {
        let result = sqlx::query!(
            r#"
                INSERT INTO youtube_raw_videos (video_id, raw_metadata)
                VALUES (?, ?)
                ON DUPLICATE KEY UPDATE
                                     raw_metadata = VALUES(raw_metadata),
                                     updated_at = CURRENT_TIMESTAMP
            "#,
            raw_video.video_id,
            raw_video.raw_metadata
        )
            .execute(&self.db_pool)
            .await?;
        
        Ok(result.last_insert_id() as i64)
    }
    
    async fn save_many(&self, raw_videos: &[YoutubeRawVideo]) -> Result<(), Error> {
        if raw_videos.is_empty() {
            return Ok(());
        }
        
        let mut query_builder = String::from(
            r#"INSERT INTO youtube_raw_videos (video_id, raw_metadata) VALUES"#
        );
        for (i, _) in raw_videos.iter().enumerate() {
            if i > 0 {
                query_builder.push_str(", ");
            }
            query_builder.push_str("(?, ?)");
        }
        query_builder.push_str(r#" ON DUPLICATE KEY UPDATE
        raw_metadata = VALUES(raw_metadata), updated_at = CURRENT_TIMESTAMP
        "#);
        
        let mut query = sqlx::query(&query_builder);
        for video in raw_videos {
            query = query
                .bind(&video.video_id)
                .bind(&video.raw_metadata);
        }
        
        query.execute(&self.db_pool).await?;
        
        Ok(())
    }
}