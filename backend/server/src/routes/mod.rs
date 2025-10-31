use actix_web::web;
use crate::handlers::user_handler::register_user;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register_user); 
}
