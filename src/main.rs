use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
}

// Handler untuk endpoint GET /hello
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
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
    HttpServer::new(|| {
        App::new()
            .route("/hello", web::get().to(hello))
            .route("/users/{id}", web::get().to(get_user))
            .route("/users", web::post().to(create_user))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}