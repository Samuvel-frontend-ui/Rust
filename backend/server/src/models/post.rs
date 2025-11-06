use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Queryable, Serialize, Selectable)]
#[diesel(table_name = crate::schema::user_posts)]
pub struct UserPost {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub description: String,
    pub videos: Vec<Option<String>>,  // ✅ Changed to match schema
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::user_posts)]
pub struct NewUserPost {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub description: String,
    pub videos: Vec<Option<String>>,  // ✅ Changed to match schema
    pub created_at: Option<NaiveDateTime>,
}