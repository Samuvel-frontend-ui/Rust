use actix_multipart::Multipart;
use actix_web::{post, web, HttpResponse, Responder};
use diesel::prelude::*;
use bcrypt::{hash, DEFAULT_COST};
use futures_util::StreamExt;
use std::io::Write;
use uuid::Uuid;
use serde_json::json;
use crate::db::DbPool;
use crate::models::user::{User, NewUser};
use crate::schema::users::dsl::*;

#[post("/register")]
pub async fn register_user(
    pool: web::Data<DbPool>,
    mut payload: Multipart,
) -> impl Responder {
    let mut user_name = String::new();
    let mut user_email = String::new();
    let mut user_password = String::new();
    let mut user_address = String::new();
    let mut user_phoneno = String::new();
    let mut user_account_type = String::from("public");
    let mut profile_pic_path: Option<String> = None;

    while let Some(Ok(mut field)) = payload.next().await {
        let field_name = field.name().to_string();
        
        match field_name.as_str() {
            "name" | "email" | "password" | "address" | "phoneno" | "account_type" => {
                let mut data = Vec::new();
                while let Some(Ok(chunk)) = field.next().await {
                    data.extend_from_slice(&chunk);
                }
                let value = String::from_utf8(data).unwrap_or_default();
                
                match field_name.as_str() {
                    "name" => user_name = value,
                    "email" => user_email = value,
                    "password" => user_password = value,
                    "address" => user_address = value,
                    "phoneno" => user_phoneno = value,
                    "account_type" => user_account_type = value,
                    _ => {}
                }
            }
            "profile_pic" => {
                let filename = field
                    .content_disposition()
                    .get_filename()
                    .map(|f| format!("{}_{}", Uuid::new_v4(), f))
                    .unwrap_or_else(|| format!("{}.jpg", Uuid::new_v4()));
                
                let filepath = format!("./uploads/{}", filename);
                std::fs::create_dir_all("./uploads").ok();
                
                let mut file = std::fs::File::create(&filepath).unwrap();
                while let Some(Ok(chunk)) = field.next().await {
                    file.write_all(&chunk).unwrap();
                }
                
                profile_pic_path = Some(filename);
            }
            _ => {}
        }
    }

    let conn = &mut pool.get().expect("DB connection failed");

    // Check if email exists
    let existing = users
        .filter(email.eq(&user_email))
        .first::<User>(conn)
        .optional()
        .expect("Error checking email");

    if existing.is_some() {
        return HttpResponse::Conflict().json(json!({
            "message": "Email already exists ❌"
        }));
    }

    let hashed = hash(&user_password, DEFAULT_COST).unwrap();

    let new_user = NewUser {
        id: Uuid::new_v4(),
        name: user_name,
        email: user_email.clone(),
        password: hashed,
        address: user_address,
        phoneno: user_phoneno,
        account_type: user_account_type,
        profile_pic: profile_pic_path,
    };

    diesel::insert_into(users)
        .values(&new_user)
        .execute(conn)
        .expect("Error inserting user");

    HttpResponse::Ok().json(json!({
        "message": "Registered successfully ✅",
        "user": user_email
    }))
}
