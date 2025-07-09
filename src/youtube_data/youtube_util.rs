use std::collections::HashSet;
use reqwest::Error;
use crate::youtube_data::youtube_data_model::{SearchListResponse, VideoItem, VideoListResponse};

pub struct YoutubeClient {
    api_key: String,
    http_client: reqwest::Client,
}

impl YoutubeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http_client: reqwest::Client::new(),
        }
    }
    
    pub async fn search_popular_shorts_ids(
        &self,
        query: &str,
        page_token: Option<&str>
    ) -> Result<SearchListResponse, Error> {
        let url = "https://www.googleapis.com/youtube/v3/search";
        let mut query_params = vec![
            ("part", "id".to_string()),
            ("order", "viewCount".to_string()),
            ("videoDuration", "short".to_string()),
            ("q", query.to_string()),
            ("maxResults", "50".to_string()),
            ("regionCode", "KR".to_string()),
            ("key", self.api_key.clone()),
        ];
        
        if let Some(token) = page_token {
            query_params.push(("pageToken", token.to_string()));
        }
        
        let response = self.http_client
            .get(url)
            .query(&query_params)
            .send()
            .await?
            .json::<SearchListResponse>()
            .await?;
        
        Ok(response)
    }
    
    pub async fn get_videos_details(
        &self,
        video_ids: &[String],
    ) -> Result<Vec<VideoItem>, Error> {
        if video_ids.is_empty() {
            return Ok(vec![]);
        }
        
        let url = "https://www.googleapis.com/youtube/v3/videos";
        let ids_str = video_ids.join(",");
        
        let response = self.http_client
            .get(url)
            .query(&[
                ("part", "snippet,contentDetails,statistics,player,topicDetails".to_string()),
                ("ids", ids_str),
                ("key", self.api_key.clone()),
            ])
            .send()
            .await?
            .json::<VideoListResponse>()
            .await?;
        
        Ok(response.items)
    }
}