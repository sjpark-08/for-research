use std::error::Error;
use actix_web::{get, post, web, HttpResponse};
use mockall::Any;
use utoipa::OpenApi;
use crate::app_state::AppState;
use crate::errors::ErrorResponse;
use crate::youtube::youtube_channel::youtube_channel_model::{AnalyzeChannelRequestQuery, ChannelKeywordResponse, ChannelRequestQuery, ChannelResponse};
use crate::youtube::youtube_video::youtube_video_model::KeywordRankingResponse;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_daily_keyword_rankings,
        get_channels,
        get_channels_keyword,
        analyze_channels_keyword
    ),
    components(),
    tags(
        (name = "Youtube Data", description = "Youtube Data endpoints")
    )
)]
pub struct YoutubeApi;

pub fn youtube_api(config: &mut web::ServiceConfig) {
    config.service(get_daily_keyword_rankings)
        .service(get_channels)
        .service(get_channels_keyword)
        .service(analyze_channels_keyword);
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
    path = "/channel",
    responses(
        (
            status = 200,
            body = ChannelResponse,
            description = "get youtube channels",
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
#[get("/channel")]
pub async fn get_channels(
    state: web::Data<AppState>
) -> Result<HttpResponse, Box<dyn Error>> {
    Ok(HttpResponse::Ok().json(""))
}

#[utoipa::path(
    get,
    path = "/channel/keyword",
    responses(
        (
            status = 200,
            body = ChannelKeywordResponse,
            description = "get youtube channels' keywords",
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
#[get("/channel/keyword")]
pub async fn get_channels_keyword(
    state: web::Data<AppState>,
    query: web::Query<ChannelRequestQuery>
) -> Result<HttpResponse, Box<dyn Error>> {
    let channel_id = &query.channel_id;
    Ok(HttpResponse::Ok().json(""))
}

#[utoipa::path(
    post,
    path = "/channel/keyword",
    responses(
        (
            status = 200,
            description = "post analyze youtube channels' keywords",
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
#[post("/channel/keyword")]
pub async fn analyze_channels_keyword(
    state: web::Data<AppState>,
    query: web::Query<AnalyzeChannelRequestQuery>
) -> Result<HttpResponse, Box<dyn Error>> {
    let channel_handle = &query.channel_handle;
    Ok(HttpResponse::Ok().json(""))
}