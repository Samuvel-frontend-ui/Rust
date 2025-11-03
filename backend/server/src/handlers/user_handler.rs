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

    // DEBUG: Track all fields received
    let mut fields_received = Vec::new();

    while let Some(Ok(mut field)) = payload.next().await {
        let field_name = field.name().to_string();
        fields_received.push(field_name.clone());
        
        println!("📥 Received field: {}", field_name);
        
        match field_name.as_str() {
            "name" | "email" | "password" | "address" | "phoneno" | "account_type" => {
                let mut data = Vec::new();
                while let Some(Ok(chunk)) = field.next().await {
                    data.extend_from_slice(&chunk);
                }
                let value = String::from_utf8(data).unwrap_or_default();
                
                println!("📝 Field '{}' = '{}'", field_name, value);
                
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
                println!("🖼️  Processing profile_pic field");
                
                let content_disposition = field.content_disposition();
                let filename = content_disposition
                    .get_filename()
                    .map(|f| {
                        println!("📷 Original filename: {}", f);
                        format!("{}_{}", Uuid::new_v4(), f)
                    })
                    .unwrap_or_else(|| {
                        let default = format!("{}.jpg", Uuid::new_v4());
                        println!("📷 No filename, using default: {}", default);
                        default
                    });
                
                let filepath = format!("./uploads/{}", filename);
                println!("💾 Saving to: {}", filepath);
                
                std::fs::create_dir_all("./uploads").ok();
                
                let mut file = match std::fs::File::create(&filepath) {
                    Ok(f) => {
                        println!("✅ File created successfully");
                        f
                    },
                    Err(e) => {
                        println!("❌ Failed to create file: {}", e);
                        return HttpResponse::InternalServerError().json(json!({
                            "message": "Failed to create file"
                        }));
                    }
                };
                
                let mut bytes_written = 0;
                while let Some(Ok(chunk)) = field.next().await {
                    bytes_written += chunk.len();
                    file.write_all(&chunk).unwrap();
                }
                
                println!("✅ Wrote {} bytes", bytes_written);
                profile_pic_path = Some(filename.clone());
                println!("✅ Set profile_pic_path to: {}", filename);
            }
            _ => {
                println!("⚠️  Unknown field: {}", field_name);
            }
        }
    }

    println!("\n📊 Summary:");
    println!("Fields received: {:?}", fields_received);
    println!("profile_pic_path final value: {:?}", profile_pic_path);

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
        profile_pic: profile_pic_path.clone(),
    };

    println!("👤 NewUser struct profile_pic: {:?}", new_user.profile_pic);

    diesel::insert_into(users)
        .values(&new_user)
        .execute(conn)
        .expect("Error inserting user");

    HttpResponse::Ok().json(json!({
        "message": "Registered successfully ✅",
        "user": user_email,
        "profile_pic": new_user.profile_pic
    }))
}