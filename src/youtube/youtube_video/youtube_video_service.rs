use std::collections::HashMap;
use std::sync::Arc;
use std::error::Error;
use chrono::{TimeDelta, Timelike, Utc};
use crate::gemini::gemini_api_util::GeminiAPIClient;
use crate::youtube::youtube_data_api::youtube_data_api_model::VideoItem;
use crate::youtube::youtube_data_api::youtube_data_api_util::YoutubeDataAPIClient;
use crate::youtube::youtube_video::youtube_raw_video_repository::YoutubeRawVideoRepository;
use crate::youtube::youtube_video::youtube_video_model::{KeywordRankingResponse, RankChange, YoutubeKeyword, YoutubeKeywordRanking, YoutubeRawVideo, YoutubeVideo};
use crate::youtube::youtube_video::youtube_video_repository::YoutubeVideoRepository;

#[derive(Clone)]
pub struct YoutubeVideoService {
    youtube_data_api_client: Arc<YoutubeDataAPIClient>,
    youtube_raw_video_repository: Arc<dyn YoutubeRawVideoRepository>,
    youtube_video_repository: Arc<dyn YoutubeVideoRepository>,
    gemini_api_client: Arc<GeminiAPIClient>,
}

impl YoutubeVideoService {
    pub fn new(
        youtube_data_api_client: Arc<YoutubeDataAPIClient>,
        youtube_raw_video_repository: Arc<dyn YoutubeRawVideoRepository>,
        youtube_video_repository: Arc<dyn YoutubeVideoRepository>,
        gemini_api_client: Arc<GeminiAPIClient>,
    ) -> Self {
        Self {
            youtube_data_api_client,
            youtube_raw_video_repository,
            youtube_video_repository,
            gemini_api_client,
        }
    }
    
    pub async fn run_video_collection_pipeline(&self) -> Result<(), Box<dyn Error>> {
        let raw_video_items = self.fetch_video_items_from_data_api().await?;
        
        let video_items = self.filter_raw_video_data(raw_video_items).await?;
        
        self.save_raw_video_data(&video_items).await?;
        
        self.transform_and_save_video_data(&video_items).await?;
        
        self.calculate_and_store_daily_rankings().await?;

        Ok(())
    }
    
    async fn fetch_video_items_from_data_api(&self) -> Result<Vec<VideoItem>, Box<dyn Error>> {
        let search_tags = vec!["#shorts", "#쇼츠"];
        let search_query = search_tags.join("|");
        let mut video_ids = Vec::new();
        
        for day in 0..8 {
            let now = Utc::now();
            let end_time = now - TimeDelta::days(day);
            let start_time = now - TimeDelta::days(day + 1);
            let mut next_page_token: Option<String> = None;
            const MAX_PAGES_TO_FETCH: u32 = 10;
            
            for _page_num in 0..MAX_PAGES_TO_FETCH {
                let response = self.youtube_data_api_client
                                   .search_popular_shorts_ids(
                                       &search_query,
                                       start_time,
                                       end_time,
                                       next_page_token.as_deref()
                                   )
                                   .await?;
                let ids: Vec<String> = response.items.into_iter().map(|item| item.id.video_id).collect();
                video_ids.extend(ids);
                
                if let Some(token) = response.next_page_token {
                    next_page_token = Some(token);
                } else {
                    break;
                }
                println!("{}", _page_num);
            }
        }
        
        let mut detailed_videos = Vec::new();
        for chunk in video_ids.chunks(50) {
            if chunk.is_empty() { break; }
            let details = self.youtube_data_api_client.get_videos_details(&chunk).await?;
            detailed_videos.extend(details);
        }
        println!("{}", detailed_videos.len());
        
        Ok(detailed_videos)
    }
    
    async fn filter_raw_video_data(&self, videos: Vec<VideoItem>) -> Result<Vec<VideoItem>, Box<dyn Error>> {
        let final_shorts: Vec<VideoItem> = videos
            .into_iter()
            .filter(|video| {
                let duration = video.content_details.as_seconds();
                let title_has_korean = video.snippet.has_korean();
                duration > 10 && duration <= 61 && title_has_korean
            })
            .collect();
        
        Ok(final_shorts)
    }
    
    async fn transform_and_save_video_data(&self, videos: &[VideoItem]) -> Result<(), Box<dyn Error>> {
        for video_chunk in videos.chunks(50) {
            let videos_to_save: Vec<YoutubeVideo> = video_chunk
                .iter()
                .map(YoutubeVideo::from)
                .collect();
            
            let keyword_map = self.gemini_api_client.extract_keywords_with_gemini(&videos_to_save).await?;
            for video in videos_to_save {
                let keywords: Vec<YoutubeKeyword> = keyword_map
                    .get(video.video_id.as_str())
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|keyword| YoutubeKeyword { id: 0, keyword_text: keyword })
                    .collect();
                
                if let Err(e) = self.save_video_and_keywords(video, keywords).await {
                    eprintln!("[Load] 개별 데이터 저장 실패: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn save_raw_video_data(&self, videos: &[VideoItem]) -> Result<(), Box<dyn Error>> {
        let raw_videos_to_save: Vec<YoutubeRawVideo> = videos
            .iter()
            .map(YoutubeRawVideo::from)
            .collect();
        
        if !raw_videos_to_save.is_empty() {
            self.youtube_raw_video_repository.save_many(&raw_videos_to_save).await?;
        }
        
        Ok(())
    }
    
    async fn save_video_and_keywords(&self, video: YoutubeVideo, keywords: Vec<YoutubeKeyword>) -> Result<(), Box<dyn Error>> {
        self.youtube_video_repository.save_video_and_keywords(video, keywords).await?;
        
        Ok(())
    }
    
    async fn calculate_and_store_daily_rankings(&self) -> Result<(), Box<dyn Error>> {
        let today = Utc::now().date_naive();
        let one_week_ago = Utc::now() - TimeDelta::days(7);
        let start_of_day = one_week_ago
            .with_hour(0).unwrap()
            .with_minute(0).unwrap()
            .with_second(0).unwrap()
            .with_nanosecond(0).unwrap();
        
        let trends = self.youtube_video_repository.get_keyword_trends(start_of_day, 50).await?;
        
        let rankings_to_save: Vec<YoutubeKeywordRanking> = trends
            .into_iter()
            .enumerate()
            .map(|(index, trend)| {
                YoutubeKeywordRanking {
                    id: 0,
                    ranking_date: today,
                    ranking: (index + 1) as i32,
                    keyword_id: trend.id,
                    keyword_text: trend.keyword_text,
                    score: trend.total_views.unwrap_or(0),
                }
            })
            .collect();
        println!("{:?}", rankings_to_save);
        self.youtube_video_repository.save_keyword_rankings(&rankings_to_save).await?;
        
        Ok(())
    }

    pub async fn get_daily_rankings(&self) -> Result<Vec<KeywordRankingResponse>, Box<dyn Error>> {
        let today = Utc::now().date_naive();
        let yesterday = today - TimeDelta::days(1);
        const RANKING_LIMIT: u32 = 50;
        
        let today_rankings = self.youtube_video_repository.get_keyword_rankings(today, RANKING_LIMIT).await?;
        let yesterday_rankings = self. youtube_video_repository.get_keyword_rankings(yesterday, RANKING_LIMIT).await?;
        
        let yesterday_rankings_map: HashMap<String, i32> = yesterday_rankings
            .into_iter()
            .map(|rank| (rank.keyword_text, rank.ranking))
            .collect();
        
        let rankings = today_rankings
            .into_iter()
            .map(|today_rank| {
                let rank_change = match yesterday_rankings_map.get(&today_rank.keyword_text) {
                    Some(yesterday_rank_value) => {
                        let diff = yesterday_rank_value - today_rank.ranking;
                        if diff > 0 {
                            RankChange::Up(diff)
                        } else if diff < 0 {
                            RankChange::Down(-diff)
                        } else {
                            RankChange::Same
                        }
                    },
                    None => RankChange::New
                };
                
                KeywordRankingResponse::from((today_rank, rank_change))
            })
            .collect();
        
        Ok(rankings)
    }
}