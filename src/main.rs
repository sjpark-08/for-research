mod user;
mod app_state;
mod config;

use actix_web::{web, App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::app_state::AppState;
use crate::user::user_controller::UserApi;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/api/v1/user", api = UserApi),
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
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
            .service(web::scope("/api/v1/user").configure(user::user_controller::user_api))
    })
        .bind(&config.server_address)?
        .run()
        .await
}