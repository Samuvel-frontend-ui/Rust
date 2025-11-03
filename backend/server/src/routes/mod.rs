use actix_web::web;
use crate::db::DbPool;

pub mod user_route;

pub fn init(cfg: &mut web::ServiceConfig, pool: DbPool, jwt_secret: String) {
    user_route::init(cfg, pool, jwt_secret);
}

