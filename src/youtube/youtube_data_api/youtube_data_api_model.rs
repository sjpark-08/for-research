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