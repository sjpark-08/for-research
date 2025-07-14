use std::sync::Arc;
use crate::youtube::youtube_data_api::youtube_data_api_model::VideoItem;
use crate::youtube::youtube_data_api::youtube_data_api_util::YoutubeDataAPIClient;
use crate::youtube::youtube_video::youtube_raw_video_repository::YoutubeRawVideoRepository;
use crate::youtube::youtube_video::youtube_video_model::{YoutubeKeyword, YoutubeRawVideo, YoutubeVideo};
use crate::youtube::youtube_video::youtube_video_repository::YoutubeVideoRepository;

#[derive(Clone)]
pub struct YoutubeVideoService {
    youtube_data_api_client: Arc<YoutubeDataAPIClient>,
    youtube_raw_video_repository: Arc<dyn YoutubeRawVideoRepository>,
    youtube_video_repository: Arc<dyn YoutubeVideoRepository>,
}

impl YoutubeVideoService {
    pub fn new(
        youtube_data_api_client: Arc<YoutubeDataAPIClient>,
        youtube_raw_video_repository: Arc<dyn YoutubeRawVideoRepository>,
        youtube_video_repository: Arc<dyn YoutubeVideoRepository>,
    ) -> Self {
        Self {
            youtube_data_api_client,
            youtube_raw_video_repository,
            youtube_video_repository,
        }
    }
    
    pub async fn run_video_collection_pipeline(&self) -> Result<(), Box<dyn std::error::Error>> {
        let raw_video_items = self.fetch_video_items_from_data_api().await?;
        println!("[Extract] {}개의 후보 영상 상세 정보 수집 완료", raw_video_items.len());
        
        let video_items = self.filter_raw_video_data(raw_video_items).await?;
        println!("[Extract] {}개의 영상으로 필터링 완료", video_items.len());
        
        self.save_raw_video_data(&video_items).await?;
        
        self.transform_and_save_video_data(&video_items).await?;
        
        Ok(())
    }
    
    async fn fetch_video_items_from_data_api(&self) -> Result<Vec<VideoItem>, Box<dyn std::error::Error>> {
        let search_tags = vec!["#short", "#shorts", "#shortvideo", "#shortsvideo"];
        let search_query = search_tags.join("|");
        let mut video_ids = Vec::new();
        let mut next_page_token: Option<String> = None;
        
        let response = self.youtube_data_api_client
                           .search_popular_shorts_ids(&search_query, None)
                           .await?;
        let ids: Vec<String> = response.items.into_iter().map(|item| item.id.video_id).collect();
        video_ids.extend(ids);
        
        let mut detailed_videos = Vec::new();
        let details = self.youtube_data_api_client.get_videos_details(&video_ids).await?;
        detailed_videos.extend(details);
        
        Ok(detailed_videos)
    }
    
    async fn filter_raw_video_data(&self, videos: Vec<VideoItem>) -> Result<Vec<VideoItem>, Box<dyn std::error::Error>> {
        let final_shorts: Vec<VideoItem> = videos
            .into_iter()
            .filter(|video| {
                let duration = video.content_details.as_seconds();
                duration > 10 && duration <= 61
            })
            .collect();
        
        Ok(final_shorts)
    }
    
    async fn transform_and_save_video_data(&self, videos: &[VideoItem]) -> Result<(), Box<dyn std::error::Error>> {
        let videos_to_save: Vec<YoutubeVideo> = videos
            .iter()
            .map(YoutubeVideo::from)
            .collect();
        
        for video in videos_to_save {
            todo!()
        }
        
        Ok(())
    }
    
    async fn save_raw_video_data(&self, videos: &[VideoItem]) -> Result<(), Box<dyn std::error::Error>> {
        let raw_videos_to_save: Vec<YoutubeRawVideo> = videos
            .iter()
            .map(YoutubeRawVideo::from)
            .collect();
        
        if !raw_videos_to_save.is_empty() {
            self.youtube_raw_video_repository.save_many(&raw_videos_to_save).await?;
        }
        
        Ok(())
    }
    
    async fn save_video_and_keywords(&self, video: YoutubeVideo, keywords: Vec<YoutubeKeyword>) -> Result<(), Box<dyn std::error::Error>> {
        self.youtube_video_repository.save_video_and_keywords(video, keywords).await?;
        
        Ok(())
    }
}