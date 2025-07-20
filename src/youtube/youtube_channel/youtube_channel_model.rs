use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct YoutubeChannel {
    pub id: i64,
    pub channel_id: String,
    pub channel_handle: String,
    pub channel_title: String,
    pub description: String,
    pub subscriber_count: i64,
    pub view_count: i64,
    pub video_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct YoutubeChannelKeyword {
    pub id: i64,
    pub youtube_channel_id: i64,
    pub keyword_text: String,
    pub view_count: i64,
}

#[derive(Serialize, Debug, Clone, ToResponse, ToSchema)]
pub struct ChannelResponse {
    pub channel_id: String,
    pub channel_handle: String,
    pub channel_title: String,
    pub thumbnail_url: String,
    pub description: String,
    pub subscriber_count: i64,
    pub view_count: i64,
    pub video_count: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Debug, Clone, ToResponse, ToSchema)]
pub struct ChannelKeywordResponse {
    pub keyword_text: String,
    pub view_count: i64,
}

#[derive(Deserialize)]
pub struct ChannelRequestQuery {
    pub channel_id: String,
}

#[derive(Deserialize)]
pub struct AnalyzeChannelRequestQuery {
    pub channel_handle: String,
}