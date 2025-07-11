use chrono::{TimeDelta, Utc};
use crate::config::Config;
use crate::youtube::youtube_data_api::youtube_data_api_error::YoutubeDataAPIError;
use crate::youtube::youtube_data_api::youtube_data_api_model::{SearchListResponse, VideoItem, VideoListResponse};

#[derive(Clone)]
pub struct YoutubeDataAPIClient {
    api_key: String,
    http_client: reqwest::Client,
}

impl YoutubeDataAPIClient {
    pub fn new(config: &Config) -> Self {
        Self {
            api_key: config.google_api_key.clone(),
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn search_popular_shorts_ids(
        &self,
        query: &str,
        page_token: Option<&str>
    ) -> Result<SearchListResponse, YoutubeDataAPIError> {
        let url = "https://www.googleapis.com/youtube/v3/search";
        
        let two_week_ago = Utc::now() - TimeDelta::weeks(4);
        let published_after_str = two_week_ago.to_rfc3339();
        
        let mut query_params: Vec<(&str, String)> = Vec::new();
        query_params.push(("part", "id".to_string()));
        // query_params.push(("order", "viewCount".to_string()));
        query_params.push(("publishedAfter", published_after_str));
        query_params.push(("videoDuration", "short".to_string()));
        query_params.push(("type", "video".to_string()));
        query_params.push(("q", "#쇼츠".to_string()));
        query_params.push(("maxResults", "50".to_string()));
        query_params.push(("regionCode", "KR".to_string()));
        query_params.push(("relevanceLanguage", "ko".to_string()));
        query_params.push(("key", self.api_key.clone()));

        if let Some(token) = page_token {
            query_params.push(("pageToken", token.to_string()));
        }

        let response = self.http_client
            .get(url)
            .query(&query_params)
            .send()
            .await?
            .error_for_status()?
            .json::<SearchListResponse>()
            .await?;
        Ok(response)
    }

    pub async fn get_videos_details(
        &self,
        video_ids: &[String],
    ) -> Result<Vec<VideoItem>, YoutubeDataAPIError> {
        if video_ids.is_empty() {
            return Ok(vec![]);
        }

        let url = "https://www.googleapis.com/youtube/v3/videos";
        let ids_str = video_ids.join(",");

        let response = self.http_client
            .get(url)
            .query(&[
                ("part", "snippet,contentDetails,statistics,player,topicDetails".to_string()),
                ("id", ids_str),
                ("key", self.api_key.clone()),
            ])
            .send()
            .await?
            .json::<VideoListResponse>()
            .await?;

        Ok(response.items)
    }
}