use utoipa::OpenApi;
use crate::handlers::user_handler;

#[derive(OpenApi)]
#[openapi(
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
        user_handler::handle_follow_request
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
            crate::models::user::NewFollow,
            crate::models::user::UserUpdate,
            crate::models::user::PaginationParams,
        )
    ),
    modifiers(&ApiDocModifier)
)]
pub struct ApiDoc;
pub struct ApiDocModifier;

impl utoipa::Modify for ApiDocModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.info.title = "My Social Media API".to_string();
        openapi.info.version = "1.0.0".to_string();
        openapi.info.description = Some("API for user management and posts".to_string());
    }
}
