use actix_web::web;
use crate::db::DbPool;

pub mod user_route;

pub fn init(cfg: &mut web::ServiceConfig, pool: DbPool, jwt_secret: String) {
    cfg.service(
        web::scope("/api")
            .configure(|scope_cfg| user_route::init(scope_cfg, pool.clone(), jwt_secret.clone())),
    );
}


