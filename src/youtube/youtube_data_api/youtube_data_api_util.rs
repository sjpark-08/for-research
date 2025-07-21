use chrono::{DateTime, Utc};
use crate::youtube::youtube_data_api::youtube_data_api_error::YoutubeDataAPIError;
use crate::youtube::youtube_data_api::youtube_data_api_model::{ChannelItem, ChannelListResponse, ChannelSearchResponse, PlaylistItemListResponse, SearchListResponse, VideoItem, VideoListResponse};

#[derive(Clone)]
pub struct YoutubeDataAPIClient {
    api_key: String,
    http_client: reqwest::Client,
}

impl YoutubeDataAPIClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn search_popular_shorts_ids(
        &self,
        query: &str,
        published_after: DateTime<Utc>,
        published_before: DateTime<Utc>,
        page_token: Option<&str>
    ) -> Result<SearchListResponse, YoutubeDataAPIError> {
        let url = "https://www.googleapis.com/youtube/v3/search";
        
        let published_after_str = published_after.to_rfc3339();
        let published_before_str = published_before.to_rfc3339();
        
        let mut query_params: Vec<(&str, String)> = Vec::new();
        query_params.push(("part", "id".to_string()));
        // query_params.push(("order", "date".to_string()));
        query_params.push(("publishedAfter", published_after_str));
        query_params.push(("publishedBefore", published_before_str));
        query_params.push(("videoDuration", "short".to_string()));
        query_params.push(("type", "video".to_string()));
        query_params.push(("q", query.to_string()));
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
        
        let mut query_params: Vec<(&str, String)> = Vec::new();
        query_params.push(("part", "snippet,contentDetails,statistics,player,topicDetails".to_string()));
        query_params.push(("id", ids_str));
        query_params.push(("key", self.api_key.clone()));

        let response = self.http_client
            .get(url)
            .query(&query_params)
            .send()
            .await?
            .json::<VideoListResponse>()
            .await?;

        Ok(response.items)
    }
    
    pub async fn find_channel_id_by_handle(&self, handle: &str) -> Result<Option<String>, YoutubeDataAPIError> {
        let url = "https://www.googleapis.com/youtube/v3/search";
        
        let mut query_params: Vec<(&str, String)> = Vec::new();
        query_params.push(("part", "snippet".to_string()));
        query_params.push(("q", handle.to_string()));
        query_params.push(("type", "channel".to_string()));
        query_params.push(("maxResults", "1".to_string()));
        query_params.push(("key", self.api_key.clone()));
        println!("시작");
        let response = self.http_client
            .get(url)
            .query(&query_params)
            .send()
            .await?
            .error_for_status()?
            .json::<ChannelSearchResponse>()
            .await?;
        println!("종료");
        // for item in response.items {
        //     println!("{}", &item.snippet.custom_url);
        //     if item.snippet.custom_url.eq_ignore_ascii_case(handle) {
        //         return Ok(Some(item.id.channel_id))
        //     }
        // }
        
        let channel_id = response.items
            .into_iter()
            .next()
            .map(|item| item.id.channel_id);
        
        Ok(channel_id)
    }
    
    pub async fn get_channel_details_by_handle(&self, channel_handle: &str) -> Result<Option<ChannelItem>, YoutubeDataAPIError> {
        let url = "https://www.googleapis.com/youtube/v3/channels";
        let mut query_params: Vec<(&str, String)> = Vec::new();
        query_params.push(("part", "snippet,contentDetails,statistics".to_string()));
        query_params.push(("forHandle", channel_handle.to_string()));
        query_params.push(("key", self.api_key.clone()));
        
        let response = self.http_client
            .get(url)
            .query(&query_params)
            .send()
            .await?
            .error_for_status()?
            .json::<ChannelListResponse>()
            .await?;
        
        let channel_item = response.items
            .into_iter()
            .next();
        
        Ok(channel_item)
    }
    
    pub async fn get_video_ids_from_playlist(&self, playlist_id: &str) -> Result<Vec<String>, YoutubeDataAPIError> {
        let url = "https://www.googleapis.com/youtube/v3/playlistItems";
        let mut video_ids = Vec::new();
        let mut next_page_token: Option<String> = None;
        
        loop {
            let mut query_params: Vec<(&str, String)> = Vec::new();
            query_params.push(("part", "contentDetails".to_string()));
            query_params.push(("playlistId", playlist_id.to_string()));
            query_params.push(("maxResults", "50".to_string()));
            query_params.push(("key", self.api_key.clone()));
            
            if let Some(token) = &next_page_token {
                query_params.push(("pageToken", token.to_string()));
            }
            
            let response = self.http_client
                .get(url)
                .query(&query_params)
                .send()
                .await?
                .error_for_status()?
                .json::<PlaylistItemListResponse>()
                .await?;
            
            let ids = response.items
                .into_iter()
                .map(|item| item.content_details.video_id);
            video_ids.extend(ids);
            
            if let Some(token) = response.next_page_token {
                next_page_token = Some(token);
            } else {
                break;
            }
        }
        
        Ok(video_ids)
    }
}