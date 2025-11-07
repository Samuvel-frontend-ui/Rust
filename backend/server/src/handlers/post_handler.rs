use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, Error, HttpRequest};
use actix_web::HttpMessage;
use serde::Serialize;
use futures_util::TryStreamExt as _;
use uuid::Uuid;
use diesel::prelude::*;
use std::fs;
use std::io::Write;
use chrono::{NaiveDateTime, Utc, Duration};
use crate::models::post::{NewUserPost, UserPostWithUser ,UserPostResponse, UserPost};
use crate::models::user::User;
use crate::schema::user_posts;
use crate::DbPool;
use utoipa::path;



#[utoipa::path(
    post,
    path = "/api/user/auth/posts",
    request_body(
        content = NewUserPost,
        content_type = "multipart/form-data",
        description = "Multipart form data with description and videos"
    ),
    responses(
        (status = 201, description = "Post uploaded successfully"),
        (status = 400, description = "Bad request: missing description or videos"),
        (status = 401, description = "Unauthorized user")
    ),
    tag = "Posts",
     security(
        ("bearerAuth" = [])
    )
)]

pub async fn create_user_post(
    pool: web::Data<DbPool>,
    mut payload: Multipart,
    req: HttpRequest,) -> Result<HttpResponse, Error> {
    let user = match req.extensions().get::<User>() {
        Some(u) => u.clone(),
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "Unauthorized"
            })));
        }
    };

    let mut description = String::new();
    let mut video_paths: Vec<String> = Vec::new();

    let upload_dir = "./files/userpost";
    fs::create_dir_all(upload_dir).expect("Failed to create upload directory");

    while let Ok(Some(mut field)) = payload.try_next().await {
        let cd = field.content_disposition().clone();
        let field_name = cd.get_name().unwrap_or_default();

        if field_name == "description" {
            // Text field
            while let Some(chunk) = field.try_next().await? {
                description.push_str(&String::from_utf8_lossy(&chunk));
            }
        } else if field_name == "videos" {
            // File upload
            let original_filename = cd
                .get_filename()
                .map(|f| f.to_string())
                .unwrap_or_else(|| "file.mp4".to_string());

            let filename = format!("{}_{}", Uuid::new_v4(), original_filename);
            let filepath = format!("{}/{}", upload_dir, filename);

            let filepath_clone = filepath.clone();
            let mut f = web::block(move || std::fs::File::create(filepath_clone)).await??;

            while let Some(chunk) = field.try_next().await? {
                f = web::block(move || {
                    f.write_all(&chunk).map(|_| f)
                })
                .await??;
            }

            let public_path = format!("{}", filename);
            video_paths.push(public_path);
        }
    }

    if description.is_empty() || video_paths.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "message": "Description and videos are required."
        })));
    }

    let conn = &mut pool.get().expect("Couldn't get DB connection");

    let videos_with_option: Vec<Option<String>> = video_paths
        .iter()
        .map(|s| Some(s.clone()))
        .collect();

    let new_post = NewUserPost {
        id: Uuid::new_v4(),
        user_id: Some(user.id),
        description: description.clone(),
        videos: videos_with_option,
        created_at: Some(Utc::now().naive_utc()),
    };

    diesel::insert_into(user_posts::table)
        .values(&new_post)
        .execute(conn)
        .expect("Failed to insert post");

    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Post uploaded successfully!",
        "post": {
            "id": new_post.id,
            "user_id": new_post.user_id,
            "description": new_post.description,
            "videos": video_paths,
            "created_at": new_post.created_at
        }
    })))
}

#[utoipa::path(
    post,
    path = "/api/user/auth/getpost",
    request_body(
        content = Multipart,
        description = "Multipart form data with description and videos"
    ),
    responses(
        (status = 201, description = "Post uploaded successfully", body = PostResponse),
        (status = 400, description = "Bad request: missing description or videos"),
        (status = 401, description = "Unauthorized user")
    ),
    tag = "Posts",
    security(
        ("bearerAuth" = [])
    )
)]

pub async fn get_user_posts(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let conn = &mut pool.get().expect("Couldn't get DB connection");

    use crate::schema::user_posts::dsl as post_dsl;
    use crate::schema::users::dsl as user_dsl;

    // Perform the join query
    let results = post_dsl::user_posts
        .left_join(user_dsl::users.on(post_dsl::user_id.eq(user_dsl::id.nullable())))
        .select((
            post_dsl::id,
            post_dsl::user_id,
            post_dsl::description,
            post_dsl::videos,
            post_dsl::created_at,
            user_dsl::name.nullable(),
            user_dsl::profile_pic.nullable(),
        ))
        .order(post_dsl::created_at.desc())
        .load::<(Uuid, Option<Uuid>, String, Vec<Option<String>>, Option<NaiveDateTime>, Option<String>, Option<String>)>(conn)
        .expect("Failed to fetch posts");

    // Transform the results to clean up the Option<String> in videos
    let response: Vec<UserPostResponse> = results
        .into_iter()
        .map(|(id, user_id, description, videos, created_at, user_name, profile_pic)| {
            UserPostResponse {
                id,
                user_id,
                description,
                videos: videos.into_iter().filter_map(|v| v).collect(),  // Remove None values
                created_at,
                user_name,
                profile_pic,
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

