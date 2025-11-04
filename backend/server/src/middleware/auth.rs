use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use diesel::prelude::*;
use futures_util::future::{ready, LocalBoxFuture, Ready, FutureExt};
use jsonwebtoken::{decode, DecodingKey, Validation, errors::ErrorKind};
use std::rc::Rc;

use crate::{
    db::DbPool,
    models::user::{User, Claims},
    schema::users::dsl::*,
};

pub struct AuthMiddlewareFactory {
    pub pool: DbPool,
    pub jwt_secret: String,
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            pool: self.pool.clone(),
            jwt_secret: self.jwt_secret.clone(),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
    pool: DbPool,
    jwt_secret: String,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.service);
        let pool = self.pool.clone();
        let secret = self.jwt_secret.clone();

        async move {
            // 🔐 Get Authorization header
            let auth_header = req.headers().get("Authorization")
                .ok_or_else(|| ErrorUnauthorized("Missing Authorization header"))?;

            let auth_str = auth_header.to_str().unwrap_or("");
            if !auth_str.starts_with("Bearer ") {
                return Err(ErrorUnauthorized("Invalid Authorization header format"));
            }

            // 🧾 Extract the token string
            let token = auth_str.trim_start_matches("Bearer").trim();

            
            // 🔍 Decode JWT token with detailed error logging
            let decoded = match decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_ref()),
                &Validation::default(),
            ) {
                Ok(data) => data,
                Err(err) => {
                    match *err.kind() {
                        ErrorKind::InvalidToken => println!("❌ Invalid token"),
                        ErrorKind::InvalidSignature => println!("❌ Invalid signature (wrong secret)"),
                        ErrorKind::ExpiredSignature => println!("⏰ Token expired"),
                        _ => println!("❌ JWT decode error: {:?}", err),
                    }
                    return Err(ErrorUnauthorized("Invalid or expired token"));
                }
            };

            let claims = decoded.claims;

            let mut conn = pool.get()
                .map_err(|_| actix_web::error::ErrorInternalServerError("DB connection failed"))?;

            let user = users
                .filter(email.eq(&claims.email))
                .first::<User>(&mut conn)
                .map_err(|_| ErrorUnauthorized("User not found"))?;

            // 📦 Store user in request extensions
            req.extensions_mut().insert(user);

            // ✅ Continue request processing
            let res = srv.call(req).await?;
            Ok(res)
        }
        .boxed_local()
    }
}
