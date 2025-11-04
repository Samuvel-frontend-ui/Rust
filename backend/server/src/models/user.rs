use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::NaiveDateTime;
use crate::schema::{users, password_reset_tokens, follows};

#[derive(Queryable, Serialize, Clone )]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub address: String,
    pub phoneno: String,
    pub account_type: String,
    pub profile_pic: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    #[serde(skip_deserializing)] 
    pub id: Uuid,

    pub name: String,
    pub email: String,
    pub password: String,
    pub address: String,
    pub phoneno: String,
    pub account_type: String,
    pub profile_pic: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims {
    pub email: String,
    pub id: String,
    pub name: String,
    pub exp: usize,
}

#[derive(Queryable, Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = password_reset_tokens)]
pub struct PasswordResetToken {
    pub id: i32,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Queryable, Serialize)]
pub struct UserListItem {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub account_type: String,
    pub profile_pic: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = follows)]
pub struct Follow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub target_id: Uuid,
    pub status: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[table_name = "follows"]
pub struct NewFollow {
    pub user_id: Uuid,
    pub target_id: Uuid,
    pub status: String,
}
                            
#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserProfile {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub profile_pic: Option<String>,
    pub account_type: String,
    pub phoneno: String,
    pub address: String,
}

#[derive(AsChangeset)]
#[diesel(table_name = users)]
pub struct UserUpdate {
    pub name: String,
    pub email: String,
    pub address: String,
    pub account_type: Option<String>,
    pub phoneno: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct ProfileUpdateBody {
    pub id: String,
    pub name: String,
    pub email: String,
    pub account_type: Option<String>,
    pub phoneno: Option<String>,
    pub address: String,
}