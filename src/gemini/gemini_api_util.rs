use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio_retry::Retry;
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use crate::config::Config;
use crate::gemini::gemini_api_model::{GeminiApiResponse, GeminiKeywordResponse, GeminiPromptVideoData};
use crate::youtube::youtube_video::youtube_video_model::YoutubeVideo;

#[derive(Clone)]
pub struct GeminiAPIClient {
    api_key: String,
    http_client: reqwest::Client,
}

impl GeminiAPIClient {
    pub fn new(config: &Config) -> Self {
        Self {
            api_key: config.google_api_key.clone(),
            http_client: reqwest::Client::new(),
        }
    }
    
    pub async fn extract_keywords_with_gemini(
        &self,
        videos: &[YoutubeVideo],
    ) -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
        if videos.is_empty() {
            return Ok(HashMap::new());
        }
        println!("키워드 추출 시작");
        let prompt_data: Vec<GeminiPromptVideoData> = videos
            .iter()
            .map(|video| {
                GeminiPromptVideoData {
                    video_id: &video.video_id,
                    title: &video.title,
                    description: &video.description,
                    tags: video.tags.as_deref().unwrap_or(&[]).iter().map(|s| s.as_str()).collect(),
                }
            })
            .collect();
        
        let prompt_data_str = serde_json::to_string(&prompt_data)?;
        let prompt_template = include_str!("keyword_extraction_prompt.txt");
        let prompt = prompt_template.replace("__VIDEO_DATA_PLACEHOLDER__", &prompt_data_str);
        println!("{}", prompt);
        
        let api_url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent";
        
        let retry_strategy = ExponentialBackoff::from_millis(100)
            .map(jitter)
            .take(3);
        
        let action = || async {
            let response = self.http_client
                .post(api_url)
                .header("x-goog-api-key", &self.api_key)
                .header("Content-Type", "application/json")
                .timeout(Duration::from_secs(120))
                .json(&serde_json::json!({
                    "contents": [{ "parts": [{ "text": prompt }] }],
                    "generationConfig": {
                        "temperature": 0.1,
                        "response_mime_type": "application/json",
                    }
                }))
                .send()
                .await?
                .error_for_status()?
                .json::<GeminiApiResponse>()
                .await?;
            
            let keyword_json_str = response
                .candidates
                .get(0)
                .and_then(|c| c.content.parts.get(0))
                .map(|p| p.text.as_str())
                .ok_or("Gemini - 키워드 추출 실패")?;
            println!("\n--- Gemini로부터 받은 실제 응답 ---");
            println!("{}", keyword_json_str);
            println!("------------------------------------\n");
            
            let keywords_results: Vec<GeminiKeywordResponse> = serde_json::from_str(&keyword_json_str)?;
            let keyword_map: HashMap<String, Vec<String>> = keywords_results
                .into_iter()
                .map(|res| (res.video_id, res.keywords))
                .collect();
            
            Ok(keyword_map)
        };
        
        Retry::spawn(retry_strategy, action).await
    }
}