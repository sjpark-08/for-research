use std::sync::Arc;
use crate::youtube::youtube_data_api::youtube_data_api_model::VideoItem;
use crate::youtube::youtube_data_api::youtube_data_api_util::YoutubeDataAPIClient;
use crate::youtube::youtube_video::youtube_raw_video_repository::YoutubeRawVideoRepository;
use crate::youtube::youtube_video::youtube_video_model::YoutubeRawVideo;

#[derive(Clone)]
pub struct YoutubeVideoService {
    youtube_data_api_client: Arc<YoutubeDataAPIClient>,
    youtube_raw_video_repository: Arc<dyn YoutubeRawVideoRepository>,
}

impl YoutubeVideoService {
    pub fn new(
        youtube_data_api_client: Arc<YoutubeDataAPIClient>,
        youtube_raw_video_repository: Arc<dyn YoutubeRawVideoRepository>,
    ) -> Self {
        Self {
            youtube_data_api_client,
            youtube_raw_video_repository
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
                let duration = Self::parse_duration_to_seconds(&video.content_details.duration);
                let is_vertical = Self::is_vertical(&video.player.embed_html);
                
                duration > 10 && duration <= 61 
            })
            .collect();
        
        let videos_to_save: Vec<YoutubeRawVideo> = final_shorts
            .iter()
            .map(YoutubeRawVideo::from)
            .collect();
        
        if !videos_to_save.is_empty() {
            if let Err(e) = self.youtube_raw_video_repository.save_many(&videos_to_save).await {
                eprintln!("DB 저장 실패 {}", e);
            }
        }
        println!("쇼츠 수집 및 저장 완료");
        Ok(())
    }
    
    fn parse_duration_to_seconds(duration: &str) -> i64 {
        let Some(duration) = duration.strip_prefix("PT") else { return 0 };
        let mut seconds = 0;
        let mut current_number = 0;
        
        for ch in duration.chars() {
            if ch.is_ascii_digit() {
                current_number = current_number * 10 + ch.to_digit(10).unwrap() as i64;
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
        println!("seconds: {}", seconds);
        seconds
    }
    fn is_vertical(embed_html: &str) -> bool {
        let get_value = |key: &str| -> Option<i32> {
            embed_html.split(key)
                .nth(1)?
                .split_once(|c: char| !c.is_ascii_digit())?
                .0.parse().ok()
        };
        if let (Some(width), Some(height)) = (get_value("width"), get_value("height")) {
            println!("{} {}", height, width);
            height > width
        } else {
            println!("no have");
            false
        }
    }
}