use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;
use utoipa::{ToSchema, IntoParams};



#[derive(Queryable, Serialize, Selectable, ToSchema)]
#[diesel(table_name = crate::schema::user_posts)]
pub struct UserPost {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub description: String,
    pub videos: Vec<Option<String>>,  // ✅ Changed to match schema
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Deserialize, Serialize, ToSchema)]
#[diesel(table_name = crate::schema::user_posts)]
pub struct NewUserPost {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub description: String,
     #[schema(value_type = Option<String>, format = Binary)]
    pub videos: Vec<Option<String>>,  // ✅ Changed to match schema
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Queryable, Selectable, ToSchema)]
#[diesel(table_name = crate::schema::user_posts)]
pub struct UserPostWithUser {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub description: String,
    pub videos: Vec<Option<String>>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Serialize, ToSchema)]
pub struct UserPostResponse {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub description: String,
    pub videos: Vec<String>,  // Cleaned up without Option
    pub created_at: Option<NaiveDateTime>,
    pub user_name: Option<String>,
    pub profile_pic: Option<String>,
}