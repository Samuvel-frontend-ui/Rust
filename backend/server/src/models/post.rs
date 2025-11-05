// use serde::{Deserialize, Serialize};
// use diesel::prelude::*;
// use uuid::Uuid;
// use chrono::NaiveDateTime;

// #[derive(Queryable, Serialize)]
// pub struct User {
//     pub id: Uuid,
//     pub name: String,
//     pub profile_pic: Option<String>,
// }

// #[derive(Queryable, Serialize)]
// pub struct UserPost {
//     pub id: Uuid,
//     pub user_id: Uuid, 
//     pub description: String,
//     pub videos: Vec<String>,
//     pub created_at: NaiveDateTime,
// }

// #[derive(Insertable, Deserialize)]
// #[diesel(table_name = crate::schema::user_posts)]
// pub struct NewUserPost {
//     pub user_id: Option<Uuid>, 
//     pub description: String,
//     pub videos: Vec<String>,
// }

// #[derive(Queryable, Serialize)]
// pub struct UserPostJoined {
//     pub post_id: Uuid,          
//     pub user_id: Option<Uuid>,  
//     pub description: String,   
//     pub videos: Vec<String>,   
//     pub created_at: NaiveDateTime, 
//     pub user_name_id: Uuid,      
//     pub name: String,           
//     pub profile_pic: Option<String>, 
// }
