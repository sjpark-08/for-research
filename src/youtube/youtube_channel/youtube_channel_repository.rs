use async_trait::async_trait;
use mockall::automock;
use sqlx::{Error, MySqlPool};
use crate::youtube::youtube_channel::youtube_channel_model::{YoutubeChannel, YoutubeChannelKeyword};

#[automock]
#[async_trait]
pub trait YoutubeChannelRepository: Send + Sync {
    async fn save_channel(&self, channel: YoutubeChannel) -> Result<i64, Error>;
    
    async fn save_channel_keywords(&self, keywords: Vec<YoutubeChannelKeyword>) -> Result<(), Error>;
    
    async fn get_keywords_by_channel_id_order_by_view_count(&self, channel_id: &str) -> Result<Vec<YoutubeChannelKeyword>, Error>;
    
}

#[derive(Clone)]
pub struct YoutubeChannelSqlxRepository {
    pub db_pool: MySqlPool,
}

impl YoutubeChannelSqlxRepository {
    pub fn new(db_pool: MySqlPool) -> Self { Self { db_pool } }
}

#[async_trait]
impl YoutubeChannelRepository for YoutubeChannelSqlxRepository {
    async fn save_channel(&self, channel: YoutubeChannel) -> Result<i64, Error> {
        let youtube_channel_id = sqlx::query!(
            r#"
                INSERT INTO youtube_channels (
                    channel_id, channel_handle, channel_title, description, subscriber_count, view_count, video_count
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            channel.channel_id,
            channel.channel_handle,
            channel.channel_title,
            channel.description,
            channel.subscriber_count,
            channel.view_count,
            channel.video_count
        )
            .execute(&self.db_pool)
            .await?
            .last_insert_id() as i64;
        
        Ok(youtube_channel_id)
    }
    
    async fn save_channel_keywords(&self, keywords: Vec<YoutubeChannelKeyword>) -> Result<(), Error> {
        if keywords.is_empty() {
            return Ok(());
        }
        
        let mut tx = self.db_pool.begin().await?;
        
        let mut query_builder = String::from(
            "INSERT INTO youtube_channel_keywords (youtube_channel_id, keyword_text, view_count) VALUES "
        );
        query_builder.push_str(&vec!["(?, ?, ?)"; keywords.len()].join(", "));
        
        let mut query = sqlx::query(&query_builder);
        for keyword in keywords {
            query = query
                .bind(keyword.youtube_channel_id)
                .bind(keyword.keyword_text)
                .bind(keyword.view_count);
        }
        
        query.execute(&mut *tx).await?;
        tx.commit().await?;
        
        Ok(())
    }
    
    async fn get_keywords_by_channel_id_order_by_view_count(&self, channel_id: &str) -> Result<Vec<YoutubeChannelKeyword>, Error> {
        let keywords = sqlx::query_as!(
            YoutubeChannelKeyword,
            r#"
                SELECT id, youtube_channel_id, keyword_text, view_count
                FROM youtube_channel_keywords
                WHERE youtube_channel_id = ?
                ORDER BY view_count DESC
            "#,
            channel_id
        )
            .fetch_all(&self.db_pool)
            .await?;
        
        Ok(keywords)
    }
}