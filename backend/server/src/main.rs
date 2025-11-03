mod db;
mod schema;
mod models;
mod handlers;
mod routes;

use actix_web::{App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use db::connection;

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
            .app_data(actix_web::web::Data::new(pool.clone()))
            .configure(routes::init) // ✅ Correct
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
