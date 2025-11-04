use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, HttpRequest, HttpMessage, Responder, Error};
use diesel::prelude::*;
use serde::Deserialize;
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
    UserListItem, Follow, NewFollow, UserProfile, UserUpdate, ProfileUpdateBody,
};

use crate::schema::users::dsl::{users, id, name, email, account_type, phoneno, address};
use crate::schema::users::dsl::*;
// use crate::schema::password_reset_tokens::dsl as reset_dsl;

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
                id: user.id.to_string(),
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

        let _ = diesel::insert_into(crate::schema::password_reset_tokens::table)
            .values((
                crate::schema::password_reset_tokens::user_id.eq(user.id),
                crate::schema::password_reset_tokens::token.eq(&token),
                crate::schema::password_reset_tokens::expires_at.eq(expires_at),
            ))
            .execute(conn);

        let reset_link = format!("http://localhost:5173/reset-password?token={}", token);

        let email_result = {
            let email_sender = "samuvel2k4@gmail.com";
            let smtp_username = "samuvel2k4@gmail.com";
            let smtp_password = "mbqy zkpi ybob xxyb";
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

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().json(json!({ "error": "Database connection failed" })),
    };

    let token_row = t::password_reset_tokens
        .filter(t::token.eq(&body.token))
        .filter(t::expires_at.gt(Utc::now().naive_utc()))
        .first::<PasswordResetToken>(&mut conn);

    if let Ok(reset_row) = token_row {
        let hashed = match hash(&body.new_password, 10) {
            Ok(h) => h,
            Err(_) => return HttpResponse::InternalServerError().json(json!({ "error": "Password hashing failed" })),
        };

        if diesel::update(u::users.filter(u::id.eq(reset_row.user_id)))
            .set(u::password.eq(&hashed))
            .execute(&mut conn)
            .is_err()
        {
            return HttpResponse::InternalServerError().json(json!({ "error": "Failed to update password" }));
        }

        if diesel::delete(t::password_reset_tokens.filter(t::token.eq(&body.token)))
            .execute(&mut conn)
            .is_err()
        {
            return HttpResponse::InternalServerError().json(json!({ "error": "Failed to delete token" }));
        }

        HttpResponse::Ok().json(json!({ "message": "Password reset successful" }))
    } else {
        HttpResponse::BadRequest().json(json!({ "error": "Token expired or invalid" }))
    }
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn get_users(pool: web::Data<DbPool>,req: HttpRequest,query: web::Query<PaginationParams>,) -> Result<HttpResponse, Error> {
    let extensions = req.extensions();
    let logged = match extensions.get::<User>() {
        Some(u) => u.clone(), 
        None => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "message": "Unauthorized"
            })));
        }
    };

    let mut conn = pool
        .get()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to get DB connection"))?;

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(6);
    let offset = (page - 1) * limit;
    let rows = users
        .filter(id.ne(logged.id))
        .select((id, name, email, account_type, profile_pic, created_at))
        .order(id.asc())
        .limit(limit)
        .offset(offset)
        .load::<UserListItem>(&mut conn)
        .unwrap_or_default();

    Ok(HttpResponse::Ok().json(json!({
        "page": page,
        "limit": limit,
        "users": rows
    })))
}

 #[derive(Deserialize)]
pub struct FollowBody {
    pub userId: String,
    pub targetId: String,
    pub action: String,
    pub isRequest: Option<bool>,
}

pub async fn follow_button(pool: web::Data<DbPool>, body: web::Json<FollowBody>) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().unwrap();

    let uid = match Uuid::parse_str(&body.userId) {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::BadRequest().json(json!({"success": false, "message": "Invalid userId"}))),
    };
    let tid = match Uuid::parse_str(&body.targetId) {
        Ok(t) => t,
        Err(_) => return Ok(HttpResponse::BadRequest().json(json!({"success": false, "message": "Invalid targetId"}))),
    };

    if body.action.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({"success": false, "message": "Missing or invalid fields"})));
    }

    use crate::schema::follows::dsl::*;
    use chrono::Utc;

    let rejected = follows.filter(user_id.eq(uid))
        .filter(target_id.eq(tid))
        .filter(status.eq("rejected"))
        .first::<Follow>(&mut conn)
        .optional()
        .unwrap();

    if let Some(r) = rejected {
        diesel::update(follows.filter(id.eq(r.id)))
            .set((status.eq("pending"), created_at.eq(Utc::now().naive_utc())))
            .execute(&mut conn)
            .unwrap();
        return Ok(HttpResponse::Ok().json(json!({"success": true, "message": "Follow request sent again", "status": "pending"})));
    }

    if body.action == "unfollow" {
        diesel::delete(follows.filter(user_id.eq(uid)).filter(target_id.eq(tid)))
            .execute(&mut conn)
            .unwrap();
        return Ok(HttpResponse::Ok().json(json!({"success": true, "message": "Unfollowed / Request cancelled"})));
    }

    let status_val = if body.isRequest.unwrap_or(false) { "pending" } else { "accepted" };

    let new_follow = NewFollow {
        user_id: uid,
        target_id: tid,
        status: status_val.to_string(),
    };

    let insert_res = diesel::insert_into(follows)
        .values(&new_follow)
        .on_conflict((user_id, target_id))
        .do_nothing()
        .execute(&mut conn)
        .unwrap();

    if insert_res == 0 {
        return Ok(HttpResponse::Ok().json(json!({"success": false, "message": if body.isRequest.unwrap_or(false) { "Request already sent" } else { "Already following" } })));
    }

    Ok(HttpResponse::Ok().json(json!({"success": true, "message": if body.isRequest.unwrap_or(false) { "Follow request sent" } else { "Now following" }, "status": status_val})))
}

pub async fn following(pool: web::Data<DbPool>, path: web::Path<String>) -> Result<HttpResponse, Error> {
    let user_id_str = path.into_inner();
    let mut conn = pool.get().unwrap();
    use crate::schema::follows::dsl::*;

    let uid = match Uuid::parse_str(&user_id_str) {
        Ok(u) => u,
        Err(_) => return Ok(HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid user id"}))),
    };

    let rows = follows
        .filter(user_id.eq(uid))
        .load::<Follow>(&mut conn)
        .unwrap_or_default();

    let following_list: Vec<uuid::Uuid> = rows.iter().filter(|r| r.status == "accepted").map(|r| r.target_id).collect();
    let pending: Vec<uuid::Uuid> = rows.iter().filter(|r| r.status == "pending").map(|r| r.target_id).collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "following": following_list,
        "pendingRequests": pending
    })))
}

pub async fn profile_get(pool: web::Data<DbPool>,user_id_str: web::Path<String>,) -> Result<HttpResponse, Error> {
   
    let user_id_str = user_id_str.into_inner();
    let uid = match Uuid::parse_str(&user_id_str) {
        Ok(u) => u,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Invalid user id"
            })));
        }
    };

    let mut conn = pool.get().unwrap();
    let user_opt = users
        .filter(id.eq(uid))
        .select(UserProfile::as_select())
        .first::<UserProfile>(&mut conn)
        .optional()
        .unwrap();

    if user_opt.is_none() {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "message": "User not found"
        })));
    }

    let user = user_opt.unwrap();
    let followers_cnt: i64 = crate::schema::follows::dsl::follows
        .filter(crate::schema::follows::dsl::target_id.eq(uid))
        .filter(crate::schema::follows::dsl::status.eq("accepted"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let following_cnt: i64 = crate::schema::follows::dsl::follows
        .filter(crate::schema::follows::dsl::user_id.eq(uid))
        .filter(crate::schema::follows::dsl::status.eq("accepted"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": user.id,
        "username": user.name,
        "email": user.email,
        "profile_pic": user.profile_pic,
        "accountType": user.account_type,
        "phoneNo": user.phoneno,
        "address": user.address,
        "FollowersCount": followers_cnt,
        "FollowingCount": following_cnt,
    })))
}

pub async fn profile_update(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    body: web::Json<ProfileUpdateBody>,
) -> Result<HttpResponse, Error> {
    let user_id_str = path.into_inner();

    // ensure user is updating their own profile
    if user_id_str != body.id {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "message": "You can only update your own profile"
        })));
    }

    let uid = match Uuid::parse_str(&user_id_str) {
        Ok(u) => u,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Invalid user id"
            })));
        }
    };

    let mut conn = pool.get().expect("Failed to get DB connection");

    // Prepare update payload
    let update_data = UserUpdate {
        name: body.name.clone(),
        email: body.email.clone(),
        address: body.address.clone(),
        account_type: body.account_type.clone(),
        phoneno: body.phoneno.clone(),
    };

    // Perform update
    let updated_user = diesel::update(users.filter(id.eq(uid)))
        .set(&update_data)
        .get_result::<User>(&mut conn)
        .optional()
        .map_err(|e| {
            eprintln!("DB update error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    if let Some(u) = updated_user {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "id": u.id,
            "username": u.name,
            "email": u.email,
            "profile_pic": u.profile_pic,
            "accountType": u.account_type,
            "phoneNo": u.phoneno,
            "address": u.address
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "message": "User not found"
        })))
    }
}




// pub async fn followers_list(pool: web::Data<DbPool>, web::Path(user_id_str): web::Path<String>, web::Query(info): web::Query<std::collections::HashMap<String,String>>) -> Result<HttpResponse, Error> {
//     let page: i64 = info.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
//     let limit: i64 = info.get("limit").and_then(|s| s.parse().ok()).unwrap_or(3);
//     let offset = (page -1) * limit;

//     let conn = pool.get().unwrap();
//     let uid = match Uuid::parse_str(&user_id_str) {
//         Ok(u) => u,
//         Err(_) => return Ok(HttpResponse::BadRequest().json(serde_json::json!({"message":"Invalid user id"}))),
//     };

//     let rows = diesel::sql_query("SELECT u.id,u.name AS username,u.profile_pic FROM follows f JOIN users u ON f.user_id = u.id WHERE f.target_id = $1 AND f.status='accepted' ORDER BY u.id ASC LIMIT $2 OFFSET $3")
//         .bind::<diesel::sql_types::Uuid,_>(uid)
//         .bind::<diesel::sql_types::BigInt,_>(limit)
//         .bind::<diesel::sql_types::BigInt,_>(offset)
//         .load::<(uuid::Uuid,String,Option<String>)>(&conn)
//         .unwrap_or_default();

//     Ok(HttpResponse::Ok().json(serde_json::json!({"page": page, "limit": limit, "followers": rows})))
// }

// pub async fn following_list(pool: web::Data<DbPool>, web::Path(user_id_str): web::Path<String>, web::Query(info): web::Query<std::collections::HashMap<String,String>>) -> Result<HttpResponse, Error> {
//     let page: i64 = info.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
//     let limit: i64 = info.get("limit").and_then(|s| s.parse().ok()).unwrap_or(3);
//     let offset = (page -1) * limit;

//     let conn = pool.get().unwrap();
//     let uid = match Uuid::parse_str(&user_id_str) {
//         Ok(u) => u,
//         Err(_) => return Ok(HttpResponse::BadRequest().json(serde_json::json!({"message":"Invalid user id"}))),
//     };

//     let rows = diesel::sql_query("SELECT u.id,u.name AS username,u.profile_pic FROM follows f JOIN users u ON f.target_id = u.id WHERE f.user_id = $1 AND f.status='accepted' ORDER BY u.id ASC LIMIT $2 OFFSET $3")
//         .bind::<diesel::sql_types::Uuid,_>(uid)
//         .bind::<diesel::sql_types::BigInt,_>(limit)
//         .bind::<diesel::sql_types::BigInt,_>(offset)
//         .load::<(uuid::Uuid,String,Option<String>)>(&conn)
//         .unwrap_or_default();

//     Ok(HttpResponse::Ok().json(serde_json::json!({"page": page, "limit": limit, "following": rows})))
// }

// pub async fn follow_requests(pool: web::Data<DbPool>, web::Path(user_id_str): web::Path<String>) -> Result<HttpResponse, Error> {
//     let conn = pool.get().unwrap();
//     let uid = match Uuid::parse_str(&user_id_str) {
//         Ok(u) => u,
//         Err(_) => return Ok(HttpResponse::BadRequest().json(serde_json::json!({"message":"Invalid user id"}))),
//     };

//     let rows = diesel::sql_query("SELECT f.id, f.user_id AS requesterId, u.name AS username, u.profile_pic FROM follows f JOIN users u ON f.user_id=u.id WHERE f.target_id=$1 AND f.status='pending' ORDER BY f.created_at ASC")
//         .bind::<diesel::sql_types::Uuid,_>(uid)
//         .load::<(i32,uuid::Uuid,String,Option<String>)>(&conn)
//         .unwrap_or_default();

//     Ok(HttpResponse::Ok().json(serde_json::json!({"pendingRequests": rows})))
// }

// #[derive(Deserialize)]
// pub struct HandleReqBody { pub action: String }

// pub async fn handle_follow_request(pool: web::Data<DbPool>, web::Path(request_id): web::Path<i32>, body: web::Json<HandleReqBody>, req: actix_web::HttpRequest) -> Result<HttpResponse, Error> {
//     let owner: Option<&User> = req.extensions().get::<User>();
//     if owner.is_none() { return Ok(HttpResponse::Unauthorized().json(serde_json::json!({"message":"Unauthorized"}))); }
//     let owner_id = owner.unwrap().id;

//     if !["approve","reject"].contains(&body.action.as_str()) { return Ok(HttpResponse::BadRequest().json(serde_json::json!({"message":"Invalid request"}))); }
//     let status_to = if body.action == "approve" { "accepted" } else { "rejected" };

//     let conn = pool.get().unwrap();
//     let updated = diesel::update(crate::schema::follows::dsl::follows.filter(crate::schema::follows::dsl::id.eq(request_id)).filter(crate::schema::follows::dsl::target_id.eq(owner_id)))
//         .set((crate::schema::follows::dsl::status.eq(status_to), crate::schema::follows::dsl::created_at.eq(Utc::now())))
//         .get_result::<Follow>(&conn)
//         .optional()
//         .unwrap();

//     if updated.is_none() { return Ok(HttpResponse::NotFound().json(serde_json::json!({"message":"Follow request not found"}))); }

//     Ok(HttpResponse::Ok().json(serde_json::json!({"message": if body.action=="approve" {"Request approved"} else {"Request rejected"}})))
// }                        