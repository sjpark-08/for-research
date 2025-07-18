use serde::{Deserialize, Serialize};

// 'search.list' API 구조체
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchListResponse {
    pub items: Vec<SearchResultItem>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchResultItem {
    pub id: SearchResultId,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultId {
    pub video_id: String,
}

// 'videos.list' API 구조체
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoListResponse {
    pub items: Vec<VideoItem>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoItem {
    pub id: String,
    pub snippet: Snippet,
    pub content_details: ContentDetails,
    pub statistics: Statistics,
    pub player: Player,
    #[serde(default)]
    pub topic_details: Option<TopicDetails>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub published_at: String,
    pub channel_id: String,
    pub title: String,
    pub description: String,
    pub channel_title: String,
    pub tags: Option<Vec<String>>,
    pub category_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentDetails {
    pub duration: String, // ISO 8601  "PT1M5S"
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    pub view_count: String,
    pub like_count: Option<String>,
    pub comment_count: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub embed_html: String,
}

#[derive(Debug, Deserialize, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicDetails {
    #[serde(default)]
    pub topic_categories: Vec<String>,
}

// 'search.list' (for channels) 구조체
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSearchResponse {
    pub items: Vec<ChannelSearchResultItem>,
}

#[derive(Debug, Deserialize)]
pub struct ChannelSearchResultItem {
    pub id: ChannelSearchResultId,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSearchResultId {
    pub channel_id: String,
}

// 'channels.list' API 구조체
#[derive(Debug, Deserialize)]
pub struct ChannelListResponse {
    pub items: Vec<ChannelItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelItem {
    pub id: String,
    pub snippet: ChannelSnippet,
    pub content_details: ChannelContentDetails,
    pub statistics: ChannelStatistics,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSnippet {
    pub title: String,
    pub description: String,
    pub custom_url: String,
    pub thumbnails: ThumbnailDetails,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelContentDetails {
    pub related_playlists: RelatedPlaylists,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelStatistics {
    pub view_count: String,
    #[serde(default)]
    pub subscriber_count: String,
    pub video_count: String,
}

#[derive(Debug, Deserialize)]
pub struct RelatedPlaylists {
    pub uploads: String,
}

#[derive(Debug, Deserialize)]
pub struct ThumbnailDetails {
    pub default: Thumbnail,
    pub medium: Thumbnail,
    pub high: Thumbnail,
}
#[derive(Debug, Deserialize)]
pub struct Thumbnail {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

// 'playlistItems.list' API 구조체

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItemListResponse {
    pub items: Vec<PlaylistItem>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItem {
    pub content_details: PlaylistItemContentDetails,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItemContentDetails {
    pub video_id: String,
}

impl ContentDetails {
    pub fn as_seconds(&self) -> i32 {
        let Some(duration) = self.duration.strip_prefix("PT") else { return 0 };
        let mut seconds = 0;
        let mut current_number = 0;
        
        for ch in duration.chars() {
            if ch.is_ascii_digit() {
                current_number = current_number * 10 + ch.to_digit(10).unwrap() as i32;
            } else {
                match ch {
                    'H' => {
                        seconds += current_number * 3600;
                        current_number = 0;
                    }
                    'M' => {
                        seconds += current_number * 60;
                        current_number = 0;
                    }
                    'S' => {
                        seconds += current_number;
                        current_number = 0;
                    }
                    _ => {}
                }
            }
        }
        seconds
    }
}

impl Snippet {
    pub fn has_korean(&self) -> bool {
        self.title.chars().any(|c| ('가'..='힣').contains(&c))
    }
}