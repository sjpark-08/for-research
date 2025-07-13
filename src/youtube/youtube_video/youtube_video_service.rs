use std::sync::Arc;
use crate::youtube::youtube_data_api::youtube_data_api_model::VideoItem;
use crate::youtube::youtube_data_api::youtube_data_api_util::YoutubeDataAPIClient;
use crate::youtube::youtube_video::youtube_raw_video_repository::YoutubeRawVideoRepository;
use crate::youtube::youtube_video::youtube_video_model::{YoutubeRawVideo, YoutubeVideo};
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
    
    pub async fn collect_and_save_raw_videos(&self) -> Result<(), Box<dyn std::error::Error>> {
        let search_tags = vec!["#short", "#shorts", "#shortvideo", "#shortsvideo"];
        let search_query = search_tags.join("|");
        let mut video_ids = Vec::new();
        let mut next_page_token: Option<String> = None;
        
        println!("인기 쇼츠 원본 데이터 수집 시작  query string = {}", search_query);
        
        let response = self.youtube_data_api_client
            .search_popular_shorts_ids(&search_query, None)
            .await?;
        let ids: Vec<String> = response.items.into_iter().map(|item| item.id.video_id).collect();
        video_ids.extend(ids);

        let mut detailed_videos = Vec::new();
        let details = self.youtube_data_api_client.get_videos_details(&video_ids).await?;
        detailed_videos.extend(details);
        println!("video count: {}", detailed_videos.len());

        let final_shorts: Vec<VideoItem> = detailed_videos
            .into_iter()
            .filter(|video| {
                let duration = video.content_details.as_seconds();
                duration > 10 && duration <= 61 
            })
            .collect();
        
        let raw_videos_to_save: Vec<YoutubeRawVideo> = final_shorts
            .iter()
            .map(YoutubeRawVideo::from)
            .collect();
        
        if !raw_videos_to_save.is_empty() {
            if let Err(e) = self.youtube_raw_video_repository.save_many(&raw_videos_to_save).await {
                eprintln!("DB 저장 실패 {}", e);
            }
        }
        println!("쇼츠 수집 및 저장 완료");
        
        let videos_to_save: Vec<YoutubeVideo> = final_shorts
            .iter()
            .map(YoutubeVideo::from)
            .collect();
        
        // TODO: 각 video 에서 keywords 추출 및 저장
        for video in videos_to_save {
        
        }
        
        
        Ok(())
    }
}