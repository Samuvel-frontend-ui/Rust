use actix_web::web;
use crate::handlers::user_handler;
use crate::db::DbPool;
use crate::middleware::auth::AuthMiddlewareFactory;

pub fn init(cfg: &mut web::ServiceConfig, pool: DbPool, jwt_secret: String) {

    cfg.service(
        web::scope("/api/user")
            .route("/register", web::post().to(user_handler::register_user))
            .route("/login", web::post().to(user_handler::login))
            .route("/forgot-password", web::post().to(user_handler::forgot_password))
            .route("/reset-password", web::post().to(user_handler::reset_password))
    );

    cfg.service(
        web::scope("/api/user")
            .wrap(AuthMiddlewareFactory {
                pool: pool.clone(),
                jwt_secret: jwt_secret.clone(),
            })
            .route("/users", web::get().to(user_handler::get_users))
    );
}