use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct GeminiApiResponse {
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
pub struct Candidate {
    pub content: Content,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    pub parts: Vec<Part>,
}

#[derive(Debug, Deserialize)]
pub struct Part {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct GeminiKeywordResponse {
    pub video_id: String,
    pub keywords: Vec<String>,
}

#[derive(Serialize)]
pub struct GeminiPromptVideoData<'a> {
    pub video_id: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    pub tags: Vec<&'a str>,
}