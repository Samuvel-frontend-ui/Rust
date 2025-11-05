// use actix_web::{web, HttpResponse, Error};
// use diesel::prelude::*;
// use uuid::Uuid;
// use serde::Deserialize;
// use crate::schema::{user_posts, users};
// use crate::models::post::{NewUserPost, UserPostJoined, UserPost};
// use crate::models::user::User;
// use crate::DbPool;

// #[derive(Deserialize)]
// pub struct CreatePostInput {
//     pub description: String,
//     pub videos: Vec<String>,
// }

// pub async fn create_user_post(
//     pool: web::Data<DbPool>,
//     req_user_id: web::ReqData<Uuid>, 
//     form: web::Json<CreatePostInput>,
// ) -> Result<HttpResponse, Error> {
//     use crate::schema::user_posts::dsl::*;

//     let extensions = req.extensions();
//     let user = match extensions.get::<User>() {
//     Some(u) => u,
//     None => {
//         return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
//             "message": "Unauthorized"
//         })));
//     }
//       };

//     let uid = user.id;


//     let new_post = NewUserPost {
//         user_id: uid, // Wrap in Some() for Option<Uuid>
//         description: form.description.clone(),
//         videos: form.videos.clone(),
//     };

//     let pool_clone = pool.clone();

//     let inserted_post: UserPost = web::block(move || {
//         let mut conn = pool_clone.get().expect("DB connection failed");

//         diesel::insert_into(user_posts)
//             .values(&new_post)
//             .returning(<UserPost>::as_returning())
//             .get_result(&mut conn)
//     })
//     .await
//     .map_err(|e| {
//         eprintln!("Block error: {:?}", e);
//         actix_web::error::ErrorInternalServerError("Failed to insert post")
//     })?
//     .map_err(|e| {
//         eprintln!("Insert error: {:?}", e);
//         actix_web::error::ErrorInternalServerError("Failed to insert post")
//     })?;

//     Ok(HttpResponse::Created().json(serde_json::json!({
//         "message": "Post uploaded successfully!",
//         "post": inserted_post
//     })))
// }

// pub async fn get_user_posts(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
//     use crate::schema::user_posts::dsl as up;
//     use crate::schema::users::dsl as u;

//     let pool_clone = pool.clone();

//     let posts: Vec<UserPostJoined> = web::block(move || {
//         let mut conn = pool_clone.get().expect("DB connection failed");

//         user_posts::table
//             .inner_join(users::table)
//             .select((
//                 up::id,
//                 up::user_id,
//                 up::description,
//                 up::videos,
//                 up::created_at,
//                 u::id,
//                 u::name,
//                 u::profile_pic,
//             ))
//             .order(up::created_at.desc())
//             .load::<UserPostJoined>(&mut conn)
//     })
//     .await
//     .map_err(|e| {
//         eprintln!("Block error: {:?}", e);
//         actix_web::error::ErrorInternalServerError("Failed to fetch posts")
//     })?
//     .map_err(|e| {
//         eprintln!("Fetch error: {:?}", e);
//         actix_web::error::ErrorInternalServerError("Failed to fetch posts")
//     })?;

//     Ok(HttpResponse::Ok().json(posts))
// }