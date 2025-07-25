use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use serde_json::Value as JsonValue;
use utoipa::{ToResponse, ToSchema};
use crate::youtube::youtube_data_api::youtube_data_api_model::VideoItem;

#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct YoutubeRawVideo {
    pub id: i64,
    pub video_id: String,
    pub raw_metadata: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct YoutubeVideo {
    pub id: i64,
    pub video_id: String,
    pub published_at: DateTime<Utc>,
    pub channel_id: String,
    pub title: String,
    pub description: String,
    pub channel_title: String,
    pub tags: Option<Vec<String>>,
    pub duration: i32,
    pub view_count: i64,
    pub like_count: i64,
    pub comment_count: i64,
    pub embed_html: String,
    pub topic_categories: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct YoutubeKeyword {
    pub id: i64,
    pub keyword_text: String,
}

#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct YoutubeKeywordRanking {
    pub id: i64,
    pub ranking_date: NaiveDate,
    pub ranking: i32,
    pub keyword_id: i64,
    pub keyword_text: String,
    pub score: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct KeywordTrend {
    pub id: i64,
    pub keyword_text: String,
    pub total_views: Option<i64>,
}

#[derive(Serialize, Debug, Clone, ToResponse, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct KeywordRankingResponse {
    pub ranking: i32,
    pub keyword_text: String,
    pub score: i64,
    pub rank_change: RankChange,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum RankChange {
    Up(i32),
    Down(i32),
    Same,
    New,
}
impl From<&VideoItem> for YoutubeRawVideo {
    fn from(item: &VideoItem) -> Self {
        let raw_metadata_json = serde_json::to_value(item)
            .unwrap_or_else(|_| serde_json::json!({}));
        
        Self {
            id: 0,
            video_id: item.id.clone(),
            raw_metadata: raw_metadata_json,
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }
}

impl From<&VideoItem> for YoutubeVideo {
    fn from(item: &VideoItem) -> Self {
        let duration_in_seconds = item.content_details.as_seconds();
        let published_at = item.snippet.published_at.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now());
        let view_count = item.statistics.view_count.parse().unwrap_or(0);
        let like_count = item.statistics.like_count.as_ref().map_or(0, |s| s.parse().unwrap_or(0));
        let comment_count = item.statistics.comment_count.as_ref().map_or(0, |s| s.parse().unwrap_or(0));
        
        Self {
            id: 0,
            video_id: item.id.clone(),
            published_at: published_at,
            channel_id: item.snippet.channel_id.clone(),
            title: item.snippet.title.clone(),
            description: item.snippet.description.clone(),
            channel_title: item.snippet.channel_title.clone(),
            tags: item.snippet.tags.clone(),
            duration: duration_in_seconds,
            view_count: view_count,
            like_count: like_count,
            comment_count: comment_count,
            embed_html: item.player.embed_html.clone(),
            topic_categories: item.topic_details.as_ref().map(|details| details.topic_categories.clone()),
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }
}

impl From<(YoutubeKeywordRanking, RankChange)> for KeywordRankingResponse {
    fn from((ranking_data, rank_change): (YoutubeKeywordRanking, RankChange)) -> Self {
        Self {
            ranking: ranking_data.ranking,
            keyword_text: ranking_data.keyword_text,
            score: ranking_data.score,
            rank_change: rank_change,
        }
    }
}