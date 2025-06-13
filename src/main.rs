mod controller;

use actix_web::{web, App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::controller::root;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::controller::root
    ),
    components(
        schemas()
    ),
    tags(
        (name = "Actix-Web", description = "My Actix-Web application endpoints")
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    HttpServer::new(|| {
        App::new()
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
            .service(root)
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}