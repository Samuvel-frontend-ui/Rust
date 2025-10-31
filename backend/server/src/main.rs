mod db;
mod schema;
mod models;
mod handlers;
mod routes;

use actix_web::{App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use db::connection;
use routes::config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = connection();

    println!("✅ Database connected successfully");
    println!("🚀 Server running on http://127.0.0.1:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin() 
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors) // ✅ Enable CORS here
            .app_data(actix_web::web::Data::new(pool.clone()))
            .configure(config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
