use std::error::Error;
use actix_web::{get, web, HttpResponse};
use utoipa::OpenApi;
use crate::app_state::AppState;
use crate::errors::ErrorResponse;
use crate::youtube::youtube_video::youtube_video_model::KeywordRankingResponse;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_daily_keyword_rankings,
        get_channels_keyword
    ),
    components(),
    tags(
        (name = "Youtube Data", description = "Youtube Data endpoints")
    )
)]
pub struct YoutubeApi;

pub fn youtube_api(config: &mut web::ServiceConfig) {
    config.service(get_daily_keyword_rankings);
}

#[utoipa::path(
    get,
    path = "/keyword/rankings",
    responses(
        (
            status = 200,
            body = KeywordRankingResponse,
            description = "get youtube keyword rankings",
            content_type = "application/json"
        ),
        (
            status = 400,
            body = ErrorResponse,
            description = "failed to get data",
        )
    ),
    tags = ["Youtube Data"]
)]
#[get("/keyword/rankings")]
pub async fn get_daily_keyword_rankings(
    state: web::Data<AppState>
) -> Result<HttpResponse, Box<dyn Error>> {
    let response = state.youtube_video_service.get_daily_rankings().await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/keyword/channel/{channel_id}",
    responses(
        (
            status = 200,
            body = KeywordRankingResponse,
            description = "get youtube channel's keywords",
            content_type = "application/json"
        ),
        (
            status = 400,
            body = ErrorResponse,
            description = "failed to get data",
        )
    ),
    tags = ["Youtube Data"]
)]
#[get("/keyword/channel/{channel_id}")]
pub async fn get_channels_keyword(
    state: web::Data<AppState>
) -> Result<HttpResponse, Box<dyn Error>> {
    Ok(HttpResponse::Ok().json(""))
}