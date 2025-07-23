use std::error::Error;
use actix_web::{get, post, web, HttpResponse};
use utoipa::OpenApi;
use crate::app_state::AppState;
use crate::auth::auth_model::AuthenticatedUser;
use crate::common::pagination::{Page, PaginationQuery};
use crate::errors::ErrorResponse;
use crate::youtube::youtube_channel::youtube_channel_model::{AnalyzeChannelRequestQuery, ChannelKeywordResponse, ChannelRequestQuery, ChannelResponse};
use crate::youtube::youtube_video::youtube_video_model::KeywordRankingResponse;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_daily_keyword_rankings,
        get_channels,
        get_channels_keyword,
        request_analyze_channels_keyword
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
        .service(request_analyze_channels_keyword);
}

#[utoipa::path(
    get,
    path = "/keyword/rankings",
    security(
        ("bearerAuth" = [])
    ),
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
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser
) -> Result<HttpResponse, Box<dyn Error>> {
    let response = state.youtube_video_service.get_daily_rankings().await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/channel",
    security(
        ("bearerAuth" = [])
    ),
    params(
        PaginationQuery
    ),
    responses(
        (
            status = 200,
            body = Page<ChannelResponse>,
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
    state: web::Data<AppState>,
    query: web::Query<PaginationQuery>,
    auth_user: AuthenticatedUser
) -> Result<HttpResponse, Box<dyn Error>> {
    let response = state.youtube_channel_service.get_youtube_channels(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/channel/keyword",
    security(
        ("bearerAuth" = [])
    ),
    params(
        ("channel_id" = String, Query, description = "channel's id")
    ),
    responses(
        (
            status = 200,
            body = Vec<ChannelKeywordResponse>,
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
    query: web::Query<ChannelRequestQuery>,
    auth_user: AuthenticatedUser
) -> Result<HttpResponse, Box<dyn Error>> {
    let channel_id = &query.channel_id;
    let response = state.youtube_channel_service.get_youtube_channel_keywords(channel_id).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/channel/keyword",
    security(
        ("bearerAuth" = [])
    ),
    params(
        ("channel_handle" = String, Query, description = "channel's handle")
    ),
    responses(
        (
            status = 202,
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
pub async fn request_analyze_channels_keyword(
    state: web::Data<AppState>,
    query: web::Query<AnalyzeChannelRequestQuery>,
    auth_user: AuthenticatedUser
) -> Result<HttpResponse, Box<dyn Error>> {
    let channel_handle = query.channel_handle.clone();
    let response = state.youtube_channel_service.request_analyze_youtube_channel_keywords(channel_handle).await?;
    Ok(HttpResponse::Accepted().json(response))
}