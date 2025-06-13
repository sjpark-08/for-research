use actix_web::{HttpResponse, Responder};
use actix_web::{get, post, put, patch, delete, options};

#[utoipa::path(
    get,
    path = "/test",
    responses(
        (status = 200, description = "Hello message", body = String),
    ),
    tags = ["Test"]
)]
#[get("/test")]
async fn root() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}