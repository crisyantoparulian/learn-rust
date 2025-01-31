use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::env;
use dotenv::dotenv;

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
}

// Handler untuk endpoint GET /hello
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

// Handler untuk endpoint GET /hello2
async fn hello2() -> impl Responder {
    HttpResponse::Ok().body("Hello2, world!")
}

// Handler untuk endpoint GET /users/{id}
async fn get_user(user_id: web::Path<u32>) -> impl Responder {
    let user = User {
        id: *user_id,
        name: "John Doe".to_string(),
    };
    HttpResponse::Ok().json(user)
}

// Handler untuk endpoint POST /users
async fn create_user(user: web::Json<User>) -> impl Responder {
    HttpResponse::Created().json(user.into_inner())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Get the port from the environment variable or default to 8080 if not set
    let port = env::var("PORT").unwrap_or_else(|_| "8082".to_string());

    HttpServer::new(|| {
        App::new()
            .route("/hello", web::get().to(hello))
            .route("/hello2", web::get().to(hello2))
            .route("/users/{id}", web::get().to(get_user))
            .route("/users", web::post().to(create_user))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}