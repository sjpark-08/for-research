use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mockall::automock;
use sqlx::{Error, MySqlPool};
use crate::youtube::youtube_video::youtube_video_model::{KeywordTrend, YoutubeKeyword, YoutubeKeywordRanking, YoutubeVideo};

#[automock]
#[async_trait]
pub trait YoutubeVideoRepository: Send + Sync {
    async fn save_video_and_keywords(&self, youtube_video: YoutubeVideo, keywords: Vec<YoutubeKeyword>) -> Result<(), Error>;
    
    async fn get_keyword_trends(&self, since: DateTime<Utc>, limit: u32) -> Result<Vec<KeywordTrend>, Error>;
    
    async fn save_keyword_rankings(&self, rankings: &[YoutubeKeywordRanking]) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct YoutubeVideoSqlxRepository {
    pub db_pool: MySqlPool,
}

impl YoutubeVideoSqlxRepository {
    pub fn new(db_pool: MySqlPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl YoutubeVideoRepository for YoutubeVideoSqlxRepository {
    async fn save_video_and_keywords(
        &self,
        youtube_video: YoutubeVideo,
        keywords: Vec<YoutubeKeyword>
    ) -> Result<(), Error> {
        
        if keywords.is_empty() {
            return Ok(());
        }
        
        // 트랜잭션 시작
        let mut tx = self.db_pool.begin().await?;
        
        let tags_json: Option<String> = youtube_video.tags.as_ref()
                                             .and_then(|tags_vec| serde_json::to_string(tags_vec).ok());
        let topics_json: Option<String> = youtube_video.topic_categories.as_ref()
                                               .and_then(|topics_vec| serde_json::to_string(topics_vec).ok());
        
        // 영상 정보 저장
        sqlx::query!(
            r#"
                INSERT INTO youtube_videos (
                    video_id, published_at, channel_id, title, description, channel_title,
                    tags, duration, view_count, like_count, comment_count, embed_html, topic_categories
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE
                    title = VALUES(title),
                    description = VALUES(description),
                    tags = VALUES(tags),
                    view_count = VALUES(view_count),
                    like_count = VALUES(like_count),
                    comment_count = VALUES(comment_count),
                    updated_at = CURRENT_TIMESTAMP
            "#,
            youtube_video.video_id,
            youtube_video.published_at,
            youtube_video.channel_id,
            youtube_video.title,
            youtube_video.description,
            youtube_video.channel_title,
            tags_json,
            youtube_video.duration,
            youtube_video.view_count,
            youtube_video.like_count,
            youtube_video.comment_count,
            youtube_video.embed_html,
            topics_json
        )
            .execute(&mut *tx)
            .await?;
        
        sqlx::query!(
            r#"
                DELETE yvk
                FROM youtube_video_keywords AS yvk
                JOIN youtube_videos AS yv ON yvk.video_id = yv.id
                WHERE yv.video_id = ?
            "#,
            youtube_video.video_id
        )
            .execute(&mut *tx)
            .await?;
        
        // 키워드 벌크 INSERT
        let keyword_texts: Vec<&str> = keywords.iter().map(|k| k.keyword_text.as_str()).collect();
        let mut keyword_query_builder = String::from("INSERT IGNORE INTO youtube_keywords (keyword_text) VALUES ");
        keyword_query_builder.push_str(&vec!["(?)"; keywords.len()].join(", "));
        
        let mut keyword_query = sqlx::query(&keyword_query_builder);
        for text in &keyword_texts {
            keyword_query = keyword_query.bind(text);
        }
        keyword_query.execute(&mut *tx).await?;
        
        // 저장한 키워드 ID 조회
        let placeholders = vec!["?"; keyword_texts.len()].join(", ");
        let select_query_str = format!(
            "SELECT id, keyword_text FROM youtube_keywords WHERE keyword_text IN ({})",
            placeholders
        );
        let mut select_query = sqlx::query_as::<_, (i64, String)>(&select_query_str);
        for text in &keyword_texts {
            select_query = select_query.bind(text);
        }
        
        let keyword_map: HashMap<String, i64> = select_query
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|(id, text)| (text, id))
            .collect();
        
        // 중간 테이블 관계 추가
        let mut link_query_builder = String::from("INSERT INTO youtube_video_keywords (video_id, keyword_id) VALUES ");
        let mut link_params = Vec::new();
        for keyword in keywords {
            if let Some(keyword_id) = keyword_map.get(&keyword.keyword_text) {
                link_params.push((youtube_video.id, *keyword_id));
            }
        }
        
        if !link_params.is_empty() {
            link_query_builder.push_str(&vec!["(?, ?)"; link_params.len()].join(", "));
            let mut link_query = sqlx::query(&link_query_builder);
            for (video_id, keyword_id) in &link_params {
                link_query = link_query.bind(video_id).bind(keyword_id)
            }
            link_query.execute(&mut *tx).await?;
        }
        
        tx.commit().await?;
        
        Ok(())
    }
    
    async fn get_keyword_trends(
        &self,
        since: DateTime<Utc>,
        limit: u32
    ) -> Result<Vec<KeywordTrend>, Error> {
        let trends = sqlx::query_as!(
            KeywordTrend,
            r#"
                SELECT yk.id, yk.keyword_text,
                       CAST(SUM(yv.view_count) AS SIGNED ) as "total_views"
                FROM youtube_videos AS yv
                JOIN youtube_video_keywords AS yvk ON yv.id = yvk.video_id
                JOIN youtube_keywords AS yk ON yvk.keyword_id = yk.id
                WHERE yv.updated_at >= ?
                GROUP BY yk.keyword_text
                ORDER BY total_views DESC
                LIMIT ?;
            "#,
            since,
            limit
        )
            .fetch_all(&self.db_pool)
            .await?;
        
        Ok(trends)
    }
    
    async fn save_keyword_rankings(&self, rankings: &[YoutubeKeywordRanking]) -> Result<(), Error> {
        if rankings.is_empty() {
            return Ok(());
        }
        
        let mut tx = self.db_pool.begin().await?;
        
        let mut query_builder = String::from(
          "INSERT INTO youtube_keyword_rankings (ranking_date, ranking, keyword_id, score) VALUES "  
        );
        query_builder.push_str(&vec!["(?, ?, ?, ?)"; rankings.len()].join(", "));
        
        let mut query = sqlx::query(&query_builder);
        for rank in rankings {
            query = query
                .bind(rank.ranking_date)
                .bind(rank.ranking)
                .bind(rank.keyword_id)
                .bind(rank.score);
        }
        
        query.execute(&mut *tx).await?;
        tx.commit().await?;
        
        Ok(())
    }
}