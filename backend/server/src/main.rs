mod db;
mod schema;
mod models;
mod handlers;
mod routes;
mod middleware;

use actix_web::{App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use db::connection;
use crate::middleware::auth::AuthMiddlewareFactory; // ✅ add this line

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = connection();

    println!("✅ Database connected successfully");
    println!("🚀 Server running on http://127.0.0.1:8081");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
App::new()
    .wrap(Logger::default())
    .wrap(cors)
    .wrap(AuthMiddlewareFactory {
        pool: pool.clone(),
        jwt_secret: "mysecretkey".to_string(), // replace with your actual secret
    })
    .app_data(actix_web::web::Data::new(pool.clone()))
    .configure(routes::init)

    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}


