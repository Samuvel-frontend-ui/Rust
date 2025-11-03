use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::NaiveDateTime;
use crate::schema::{users, password_reset_tokens};

#[derive(Queryable, Serialize)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub id: String,
    pub name: String,
    pub email: String,
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


#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTokenClaims {
    pub id: Uuid,
    pub exp: usize,
}

#[derive(serde::Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(serde::Serialize)]
pub struct UsersResponse {
    page: i64,
    limit: i64,
    users: Vec<User>,
}