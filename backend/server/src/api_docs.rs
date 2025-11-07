use utoipa::OpenApi;  
use utoipa::Modify;   
use utoipa::openapi::security::{SecurityScheme, HttpBuilder, HttpAuthScheme}; // JWT scheme
use crate::handlers::user_handler; 
use crate::handlers::post_handler;
use crate::models::{user, post};

#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "http://127.0.0.1:8081", description = "Local development server")
    ),
    paths(
        user_handler::register_user,
        user_handler::login,
        user_handler::forgot_password,
        user_handler::reset_password,
        user_handler::get_users,
        user_handler::follow_button,
        user_handler::profile_get,
        user_handler::profile_update,
        user_handler::followers_list,
        user_handler::following_list,
        user_handler::follow_requests,
        user_handler::handle_follow_request,
        post_handler::create_user_post,
        post_handler::get_user_posts
    ),
    components(
        schemas(
            crate::models::user::LoginRequest,
            crate::models::user::ForgotPasswordRequest,
            crate::models::user::ResetPasswordRequest,
            crate::models::user::UserUpdateRequest,
            crate::models::user::FollowBody,
            crate::models::user::UserListItem,
            crate::models::user::UserProfile,
            crate::models::user::FollowerInfo,
            crate::models::user::PendingRequest,
            crate::models::user::Followreq,
            crate::models::user::Claims,
            crate::models::user::PasswordResetToken,
            crate::models::user::NewUser,
            crate::models::user::User,
            crate::models::user::NewFollow,
            crate::models::user::UserUpdate,
            crate::models::user::PaginationParams,
            crate::models::user::HandleFollowRequest,
            crate::models::post::NewUserPost,
            crate::models::post::UserPostWithUser,
            crate::models::post::UserPostResponse,
            crate::models::post::UserPost   
        )
    ),
    modifiers(&ApiDocModifier)
)]
pub struct ApiDoc;

pub struct ApiDocModifier;

impl Modify for ApiDocModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.info.title = "My Social Media API".to_string();
        openapi.info.version = "1.0.0".to_string();
        openapi.info.description = Some("API for user management and posts".to_string());

        
        openapi.components.as_mut().unwrap().security_schemes.insert(
            "bearerAuth".to_string(),
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build()
            ),
        );
    }
}
