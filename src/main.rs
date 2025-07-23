mod user;
mod app_state;
mod config;
mod errors;
mod youtube;
mod gemini;
mod auth;
mod redis;
mod common;

use actix_web::{web, App, HttpServer};
use utoipa::{Modify, OpenApi};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa_swagger_ui::SwaggerUi;
use crate::app_state::AppState;
use crate::user::user_controller::UserApi;
use youtube::youtube_video_controller::YoutubeApi;
use crate::auth::auth_controller::AuthApi;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}
#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    nest(
        (path = "/api/v1/user", api = UserApi),
        (path = "/api/v1/auth", api = AuthApi),
        (path = "/api/v1/data/youtube", api = YoutubeApi),
    ),
    components(
        schemas(),
    ),
    info(
        title = "Actix-Web API",
        version = "0.1.0",
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    let config = config::Config::from_env();
    let app_state = AppState::new(&config).await;
    
    env_logger::init();
    youtube::youtube_video::youtube_video_scheduler::init_scheduler(app_state.clone());
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
            .service(web::scope("/api/v1/user").configure(user::user_controller::user_api))
            .service(web::scope("/api/v1/auth").configure(auth::auth_controller::auth_api))
            .service(web::scope("/api/v1/data/youtube").configure(youtube::youtube_video_controller::youtube_api))
    })
        .bind(&config.server_address)?
        .run()
        .await
}