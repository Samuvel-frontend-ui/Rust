use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use crate::schema::users;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Serialize, Deserialize)]
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
    #[serde(skip_deserializing)] // ✅ Ignore id from JSON input
    pub id: Uuid,

    pub name: String,
    pub email: String,
    pub password: String,
    pub address: String,
    pub phoneno: String,
    pub account_type: String,
    pub profile_pic: Option<String>,
}
