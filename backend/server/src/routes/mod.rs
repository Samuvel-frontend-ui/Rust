pub mod user_route;
use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/user")
            .configure(user_route::init)
    );
}
