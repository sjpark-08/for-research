use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use crate::common::pagination::{Page, PaginationQuery};
use crate::errors::AppError;
use crate::gemini::gemini_api_util::GeminiAPIClient;
use crate::youtube::youtube_channel::youtube_channel_error::YoutubeChannelError;
use crate::youtube::youtube_channel::youtube_channel_model::{ChannelKeywordResponse, ChannelResponse, YoutubeChannel, YoutubeChannelKeyword};
use crate::youtube::youtube_channel::youtube_channel_repository::YoutubeChannelRepository;
use crate::youtube::youtube_data_api::youtube_data_api_model::ChannelItem;
use crate::youtube::youtube_data_api::youtube_data_api_util::YoutubeDataAPIClient;
use crate::youtube::youtube_video::youtube_video_model::YoutubeVideo;

#[derive(Clone)]
pub struct YoutubeChannelService {
    youtube_channel_repository: Arc<dyn YoutubeChannelRepository>,
    youtube_data_api_client: Arc<YoutubeDataAPIClient>,
    gemini_api_client: Arc<GeminiAPIClient>
}

impl YoutubeChannelService {
    pub fn new(
        youtube_channel_repository: Arc<dyn YoutubeChannelRepository>,
        youtube_data_api_client: Arc<YoutubeDataAPIClient>,
        gemini_api_client: Arc<GeminiAPIClient>
    ) -> Self {
        Self {
            youtube_channel_repository,
            youtube_data_api_client,
            gemini_api_client,
        }
    }
    
    pub async fn request_analyze_youtube_channel_keywords(&self, channel_handle: String) -> Result<serde_json::Value, AppError> {
        let search_query = if channel_handle.starts_with('@') {
            channel_handle
        } else {
            format!("@{}", channel_handle)
        };
        
        if self.youtube_channel_repository.channel_exists_by_handle(&search_query).await? {
            return Err(YoutubeChannelError::ChannelDuplicated(search_query.to_string()))?;
        }
        
        let channel_details = self.youtube_data_api_client
            .get_channel_details_by_handle(&search_query)
            .await?
            .ok_or_else(|| YoutubeChannelError::ChannelNotFound(search_query.to_string()))?;
        
        let channel = YoutubeChannel::from(&channel_details);
        let youtube_channel_id = self.youtube_channel_repository
                                     .save_channel(channel)
                                     .await?;
        
        let service_clone = self.clone();
        tokio::spawn(async move {
           if let Err(e) = service_clone.analyze_youtube_channel_keywords(youtube_channel_id, channel_details).await {
               eprintln!("[Background Job] 채널 '{}' 분석 실패: {}", search_query, e);
           } else {
               println!("[Background Job] 채널 '{}' 분석 성공!", search_query);
           }
        });
        
        Ok(serde_json::json!({
            "message": "채널 분석 요청이 접수되었습니다. 분석에는 몇 분 정도 소요될 수 있습니다."
        }))
    }
    
    async fn analyze_youtube_channel_keywords(
        &self, youtube_channel_id: i64, 
        channel_details: ChannelItem
    ) -> Result<(), Box<dyn Error>> {
        let upload_playlist_id = channel_details.content_details.related_playlists.uploads.clone();
        
        let video_ids = self.youtube_data_api_client
                            .get_video_ids_from_playlist(&upload_playlist_id)
                            .await?;
        
        let mut detailed_videos = Vec::new();
        for chunk in video_ids.chunks(50) {
            let details = self.youtube_data_api_client.get_videos_details(&chunk).await?;
            detailed_videos.extend(details);
        }
        
        let mut final_keywords_map = HashMap::new();
        let mut count = 0;
        for video_chunk in detailed_videos.chunks(50) {
            count += 1;
            println!("{} 번쨰 청크", count);
            let videos: Vec<YoutubeVideo> = video_chunk
                .iter()
                .map(YoutubeVideo::from)
                .collect();
            
            let keywords_map = self.gemini_api_client
                                   .extract_keywords_with_gemini(&videos)
                                   .await?;
            
            for video in videos {
                if let Some(keywords) = keywords_map.get(&video.video_id) {
                    for keyword in keywords {
                        *final_keywords_map.entry(keyword.clone()).or_insert(0) += video.view_count;
                    }
                }
            }
        }
        
        let channel_keywords: Vec<YoutubeChannelKeyword> = final_keywords_map
            .into_iter()
            .map(|(k, v)| {
                YoutubeChannelKeyword {
                    id: Default::default(),
                    youtube_channel_id: youtube_channel_id,
                    keyword_text: k,
                    view_count: v,
                }
            })
            .collect();
        
        self.youtube_channel_repository.save_channel_keywords(channel_keywords).await?;
        
        self.youtube_channel_repository.update_channel_finished_by_id(youtube_channel_id).await?;
        
        Ok(())
    }
    
    pub async fn get_youtube_channels(&self, query: PaginationQuery) -> Result<Page<ChannelResponse>, Box<dyn Error>> {
        let limit = query.size;
        let offset = query.page * query.size;
        
        let total_items = self.youtube_channel_repository.count_all_channels().await?;
        
        let youtube_channels = self.youtube_channel_repository.find_all_channels(limit, offset).await?;
        let response = youtube_channels
            .iter()
            .map(ChannelResponse::from)
            .collect();
        
        let total_pages = (total_items as f64 / limit as f64).ceil() as u32;
        
        Ok(Page {
            items: response,
            page: query.page,
            size: query.size,
            total_items,
            total_pages
        })
    }
    
    pub async fn get_youtube_channel_keywords(&self, channel_handle: &str) -> Result<Vec<ChannelKeywordResponse>, Box<dyn Error>> {
        let youtube_channel_keywords = self.youtube_channel_repository
            .find_keywords_by_channel_handle_order_by_view_count(channel_handle, 100)
            .await?;
        let response = youtube_channel_keywords
            .iter()
            .map(ChannelKeywordResponse::from)
            .collect();
        
        Ok(response)
    }
    
    pub async fn cleanup_stale_channels(&self) -> Result<(), Box<dyn Error>> {
        self.youtube_channel_repository.delete_channel_not_finished().await?;
        
        Ok(())
    }
}