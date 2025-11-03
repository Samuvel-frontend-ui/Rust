use actix_web::web;
use crate::handlers::user_handler;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/register", web::post().to(user_handler::register_user),);
    cfg.route("/login", web::post().to(user_handler::login),);
    cfg.route("/forgot-password", web::post().to(user_handler::forgot_password),);
    cfg.route("/reset-password", web::post().to(user_handler::reset_password),);
}
