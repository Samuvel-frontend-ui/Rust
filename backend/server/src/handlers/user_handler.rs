use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, Responder};
use diesel::prelude::*;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
use lettre::{
    message::{header, Message},
    transport::smtp::authentication::Credentials,
    SmtpTransport, Transport,
};
use futures_util::StreamExt;
use std::io::Write;
use uuid::Uuid;
use serde_json::json;
use chrono::{Utc, Duration};

use crate::db::DbPool;
use crate::models::user::{
    User, NewUser, LoginRequest, ForgotPasswordRequest, ResetPasswordRequest, Claims, PasswordResetToken,
};

use crate::schema::users::dsl::*;

pub async fn register_user(pool: web::Data<DbPool>, mut payload: Multipart) -> impl Responder {
    let mut user_name = String::new();
    let mut user_email = String::new();
    let mut user_password = String::new();
    let mut user_address = String::new();
    let mut user_phoneno = String::new();
    let mut user_account_type = String::from("public");
    let mut profile_pic_filename: Option<String> = None;

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => {
                eprintln!("❌ Multipart read error: {}", e);
                return HttpResponse::BadRequest().json(json!({"message": "Invalid upload"}));
            }
        };

        let field_name = field.name().to_string();

        if ["name", "email", "password", "address", "phoneno", "account_type"]
            .contains(&field_name.as_str())
        {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                if let Ok(bytes) = chunk {
                    data.extend_from_slice(&bytes);
                }
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
        } else if field_name == "profile_pic" {
            let upload_dir = "./files/userprofile";
            if let Err(e) = std::fs::create_dir_all(upload_dir) {
                eprintln!("❌ Failed to create upload dir: {}", e);
            }

            let filename = field
                .content_disposition()
                .get_filename()
                .map(|f| format!("{}_{}", Uuid::new_v4(), f))
                .unwrap_or_else(|| format!("{}.jpg", Uuid::new_v4()));

            let filepath = format!("{}/{}", upload_dir, filename);
            match std::fs::File::create(&filepath) {
                Ok(mut file) => {
                    while let Some(chunk) = field.next().await {
                        if let Ok(data) = chunk {
                            if let Err(e) = file.write_all(&data) {
                                eprintln!("File write error: {}", e);
                            }
                        }
                    }
                    profile_pic_filename = Some(filename.clone());
                }
                Err(e) => {
                    eprintln!("❌ File create error: {}", e);
                }
            }
        }
    }

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ DB connection error: {}", e);
            return HttpResponse::InternalServerError()
                .json(json!({"message": "Database connection failed"}));
        }
    };

    if user_email.trim().is_empty() {
        return HttpResponse::BadRequest().json(json!({"message": "Email is required"}));
    }

    let existing = users
        .filter(email.eq(&user_email))
        .first::<User>(&mut conn)
        .optional()
        .unwrap_or(None);

    if existing.is_some() {
        return HttpResponse::Conflict().json(json!({"message": "Email already exists ❌"}));
    }

    let hashed = match hash(&user_password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("❌ Hash error: {}", e);
            return HttpResponse::InternalServerError()
                .json(json!({"message": "Password hashing failed"}));
        }
    };

    let new_user = NewUser {
        id: Uuid::new_v4(),
        name: user_name,
        email: user_email.clone(),
        password: hashed,
        address: user_address,
        phoneno: user_phoneno,
        account_type: user_account_type,
        profile_pic: profile_pic_filename.clone(),
    };

    if let Err(e) = diesel::insert_into(users).values(&new_user).execute(&mut conn) {
        eprintln!("❌ DB insert error: {}", e);
        return HttpResponse::InternalServerError().json(json!({"message": "Database insert failed"}));
    }

    HttpResponse::Ok().json(json!({
        "message": "Registered successfully ✅"
    }))
}

pub async fn login(pool: web::Data<DbPool>, data: web::Json<LoginRequest>) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection failed"),
    };

    let result = users.filter(email.eq(&data.email)).first::<User>(&mut conn);

    match result {
        Ok(user) => {
            // Safe password check
            let is_valid = verify(&data.password, &user.password).unwrap_or(false);

            if !is_valid {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "message": "Invalid password"
                }));
            }

            // Safe JWT creation
            let expiration = Utc::now()
                .checked_add_signed(Duration::hours(24))
                .expect("valid timestamp")
                .timestamp() as usize;

            let claims = Claims {
                sub: user.id.to_string(),
                name: user.name.clone(),
                email: user.email.clone(),
                exp: expiration,
            };

            let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "mysecretkey".into());

            let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())) {
                Ok(t) => t,
                Err(_) => return HttpResponse::InternalServerError().json("Token creation failed"),
            };

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Login successful",
                "token": token,
                "user": {
                    "id": user.id,
                    "name": user.name,
                    "email": user.email,
                    "profile_pic": user.profile_pic
                }
            }))
        }
        Err(_) => HttpResponse::Unauthorized().json(serde_json::json!({
            "message": "User not found"
        })),
    }
}

pub async fn forgot_password(pool: web::Data<DbPool>,body: web::Json<ForgotPasswordRequest>,) -> impl Responder {
    let conn = &mut pool.get().unwrap();

    let user_result = users.filter(email.eq(&body.email)).first::<User>(conn);

    if let Ok(user) = user_result {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now().naive_utc() + chrono::Duration::hours(1);

        // insert into password_reset_tokens table (use fully-qualified schema path to avoid DSL confusion)
        let _ = diesel::insert_into(crate::schema::password_reset_tokens::table)
            .values((
                crate::schema::password_reset_tokens::user_id.eq(user.id),
                crate::schema::password_reset_tokens::token.eq(&token),
                crate::schema::password_reset_tokens::expires_at.eq(expires_at),
            ))
            .execute(conn);

        let reset_link = format!("http://localhost:5173/reset-password?token={}", token);

        // avoid naming conflict with 'email' column by naming variable `mail_msg`
        let email_result = {
            let email_sender = "yourapp@example.com";
            let smtp_username = "yourapp@example.com";
            let smtp_password = "your-smtp-password";
            let smtp_server = "smtp.gmail.com";

            let mail_msg = Message::builder()
                .from(email_sender.parse().unwrap())
                .to(user.email.parse().unwrap())
                .subject("Password Reset Request")
                .header(header::ContentType::TEXT_HTML)
                .body(format!(
                    "<p>Hello, {}</p>\
                     <p>Click below to reset your password:</p>\
                     <a href=\"{}\">Reset Password</a>\
                     <p>This link expires in 1 hour.</p>",
                    user.name, reset_link
                ))
                .unwrap();

            let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());
            let mailer = SmtpTransport::relay(smtp_server).unwrap().credentials(creds).build();

            mailer.send(&mail_msg)
        };

        match email_result {
            Ok(_) => HttpResponse::Ok().json(json!({ "message": "Reset link sent successfully" })),
            Err(err) => HttpResponse::InternalServerError()
                .json(json!({ "error": format!("Email send failed: {}", err) })),
        }
    } else {
        HttpResponse::BadRequest().json(json!({ "error": "Invalid user email" }))
    }
}

pub async fn reset_password(pool: web::Data<DbPool>,body: web::Json<ResetPasswordRequest>,) -> impl Responder {
    use crate::schema::{users::dsl as u, password_reset_tokens::dsl as t};

    let conn = &mut pool.get().unwrap();

    let token_row = t::password_reset_tokens
        .filter(t::token.eq(&body.token))
        .filter(t::expires_at.gt(Utc::now().naive_utc()))
        .first::<PasswordResetToken>(conn);

    if let Ok(reset_row) = token_row {
        let hashed = hash(&body.new_password, 10).unwrap();
        diesel::update(u::users.filter(u::id.eq(reset_row.user_id)))
            .set(u::password.eq(hashed))
            .execute(conn)
            .unwrap();

        diesel::delete(t::password_reset_tokens.filter(t::token.eq(&body.token)))
            .execute(conn)
            .unwrap();

        HttpResponse::Ok().json(serde_json::json!({ "message": "Password reset successful" }))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({ "error": "Token expired or invalid" }))
    }
}




