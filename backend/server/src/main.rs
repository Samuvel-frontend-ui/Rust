pub mod db;
pub mod models;
pub mod schema;
pub mod handlers;
pub mod routes;
pub mod middleware;

use actix_web::{App, HttpServer, middleware::Logger, web};
use actix_files::Files;
use actix_cors::Cors;
use db::{DbPool, connection};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = connection();
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    println!("✅ Database connected successfully");
    println!("🚀 Server running on http://127.0.0.1:8081");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_origin("http://127.0.0.1:5173")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Content-Type", "Authorization"])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            // Your API/config/init here
            .configure(|cfg| routes::init(cfg, pool.clone(), "mysecretkey".to_string()))
            // Serve profile pictures statically
            .service(Files::new("/profile_pic", "./files/userprofile").show_files_listing())
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
