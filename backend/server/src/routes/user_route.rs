use actix_web::web;
use crate::handlers::user_handler;
// use crate::handlers::post_handler;
use crate::db::DbPool;
use crate::middleware::auth::AuthMiddlewareFactory;

pub fn init(cfg: &mut web::ServiceConfig, pool: DbPool, jwt_secret: String) {
    cfg.service(
        web::scope("/user")
            .route("/register", web::post().to(user_handler::register_user))
            .route("/login", web::post().to(user_handler::login))
            .route("/forgot-password", web::post().to(user_handler::forgot_password))
            .route("/reset-password", web::post().to(user_handler::reset_password))
            .service(
                web::scope("/auth")
                    .wrap(AuthMiddlewareFactory {
                        pool: pool.clone(),
                        jwt_secret: jwt_secret.clone(),
                    })
                    .route("/get-users", web::get().to(user_handler::get_users))
                    .route("/follow", web::post().to(user_handler::follow_button))
                    .route("/request/{user_id}", web::get().to(user_handler::following))
                    .route("/profile/{user_id}", web::get().to(user_handler::profile_get))
                    .route("/profile-update/{user_id}", web::put().to(user_handler::profile_update))
                    .route("/followers/{user_id}", web::get().to(user_handler::followers_list))
                    .route("/followings/{user_id}", web::get().to(user_handler::following_list))
                    .route("/follow-req/{user_id}", web::get().to(user_handler::follow_requests))
                    .route("/handle-follow-req/{request_id}", web::post().to(user_handler::handle_follow_request))
                    // .route("/post", web::post().to(post_handler::create_user_post))
                    // .route("/getpost", web::get().to(post_handler::get_user_posts)),
            ),
    );
}
