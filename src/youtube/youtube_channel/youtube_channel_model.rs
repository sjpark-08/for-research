use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};
use crate::youtube::youtube_data_api::youtube_data_api_model::ChannelItem;

#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct YoutubeChannel {
    pub id: i64,
    pub channel_id: String,
    pub channel_handle: String,
    pub channel_title: String,
    pub thumbnail_url: String,
    pub description: String,
    pub subscriber_count: i64,
    pub view_count: i64,
    pub video_count: i64,
    pub is_finished: bool,
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
    pub is_finished: bool,
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

impl From<&ChannelItem> for YoutubeChannel {
    fn from(item: &ChannelItem) -> Self {
        let subscriber_count = item.statistics.subscriber_count.parse().unwrap_or(0);
        let view_count = item.statistics.view_count.parse().unwrap_or(0);
        let video_count = item.statistics.video_count.parse().unwrap_or(0);
        
        Self {
            id: Default::default(),
            channel_id: item.id.clone(),
            channel_title: item.snippet.title.clone(),
            channel_handle: item.snippet.custom_url.clone(),
            thumbnail_url: item.snippet.thumbnails.default.url.clone(),
            description: item.snippet.description.clone(),
            subscriber_count: subscriber_count,
            view_count: view_count,
            video_count: video_count,
            is_finished: false,
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }
}

impl From<&YoutubeChannel> for ChannelResponse {
    fn from(item: &YoutubeChannel) -> Self {
        Self {
            channel_id: item.channel_id.clone(),
            channel_handle: item.channel_handle.clone(),
            channel_title: item.channel_title.clone(),
            thumbnail_url: item.thumbnail_url.clone(),
            description: item.description.clone(),
            subscriber_count: item.subscriber_count,
            view_count: item.view_count,
            video_count: item.video_count,
            is_finished: item.is_finished,
            updated_at: item.updated_at,
        }
    }
}

impl From<&YoutubeChannelKeyword> for ChannelKeywordResponse {
    fn from(item: &YoutubeChannelKeyword) -> Self {
        Self {
            keyword_text: item.keyword_text.clone(),
            view_count: item.view_count,
        }
    }
}