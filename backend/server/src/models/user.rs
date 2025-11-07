use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::NaiveDateTime;
use utoipa::{ToSchema, IntoParams};
use crate::schema::{users, password_reset_tokens, follows};

#[derive(Queryable, Serialize, Clone, ToSchema )]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub address: Option<String>,
    pub phoneno: String,
    pub account_type: String,
    pub profile_pic: Option<String>,
    pub created_at: NaiveDateTime,
}


#[derive(Insertable, Deserialize, ToSchema)]
#[diesel(table_name = users)]
pub struct NewUser {
    #[schema(example = "John Doe")]
    pub name: String,
    
    #[schema(example = "john@example.com")]
    pub email: String,
    
    #[schema(example = "SecurePass123!")]
    pub password: String,
    
    #[schema(example = "123 Main St, City")]
    pub address: String,
    
    #[schema(example = "+1234567890")]
    pub phoneno: String,
    
    #[schema(example = "public", default = "public")]
    pub account_type: String,
    
    #[schema(value_type = Option<String>, format = Binary)]
    pub profile_pic: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct Claims {
    pub email: String,
    pub id: String,
    pub name: String,
    pub exp: usize,
}

#[derive(Queryable, Insertable, Serialize, Deserialize, Debug, ToSchema)]
#[diesel(table_name = password_reset_tokens)]
pub struct PasswordResetToken {
    pub id: i32,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Queryable, Serialize, ToSchema)]
pub struct UserListItem {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub account_type: String,
    pub profile_pic: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Insertable, ToSchema)]
#[diesel(table_name = follows)]
pub struct Follow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub target_id: Uuid,
    pub status: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, ToSchema)]
#[table_name = "follows"]
pub struct NewFollow {
    pub user_id: Uuid,
    pub target_id: Uuid,
    pub status: String,
}
                            
#[derive(Queryable, Selectable, Serialize, ToSchema)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserProfile {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub profile_pic: Option<String>,
    pub account_type: String,
    pub phoneno: String,
    pub address: Option<String>,
}

#[derive(AsChangeset, ToSchema)]
#[diesel(table_name = users)]
pub struct UserUpdate {
    pub name: String,
    pub email: String,
    pub address: String,
    pub account_type: Option<String>,
    pub phoneno: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UserUpdateRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub accountType: Option<String>,
    pub phoneNo: Option<String>,
    pub address: Option<String>,
    pub loggedInUserId: Option<Uuid>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

 #[derive(Deserialize, ToSchema)]
pub struct FollowBody {
    pub userId: String,
    pub targetId: String,
    pub action: String,
    pub isRequest: Option<bool>,
}

#[derive(Queryable, Serialize, ToSchema)]
pub struct FollowerInfo {
    pub user_id: Uuid,
    pub name: String,
    pub profile_pic: Option<String>,
}

#[derive(Queryable, Serialize, Identifiable, ToSchema)]
#[diesel(table_name = follows)]
#[diesel(primary_key(id))]
pub struct Followreq {
    pub id: Uuid,
    pub user_id: Uuid,
    pub target_id: Uuid,
    pub status: String,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct PendingRequest {
    pub id: Uuid,
    pub requester_id: Uuid,
    pub username: String,
    pub profile_pic: Option<String>,
}


#[derive(Deserialize, ToSchema)]
pub struct HandleFollowRequest {
    pub action: String,
    pub ownerId: Option<String>, // Add this
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub message: String,
    pub token: String,
    pub user: serde_json::Value,
}