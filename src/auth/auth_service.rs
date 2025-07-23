use std::sync::Arc;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::{http, HttpRequest, HttpResponse};
use bcrypt::verify;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde_json::json;
use crate::auth::auth_error::AuthError;
use crate::auth::auth_model::{Claims, LoginRequest};
use crate::config::Config;
use crate::redis::redis_repository::RedisRepository;
use crate::user::user_repository::UserRepository;

#[derive(Clone)]
pub struct AuthService {
    pub user_repository: Arc<dyn UserRepository>,
    pub redis_repository: Arc<RedisRepository>,
    pub jwt_secret: String,
}

impl AuthService {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        redis_repository: Arc<RedisRepository>,
    ) -> Self {
        let config = Config::from_env();
        let jwt_secret = config.jwt_secret.clone();
        Self {
            user_repository,
            redis_repository,
            jwt_secret,
        }
    }
    
    pub async fn login(&self, request: LoginRequest) -> Result<HttpResponse, AuthError> {
        let user = self.user_repository.find_by_email(&request.email).await?;
        
        let is_valid = verify(&request.password, &user.password)
            .map_err(|_| AuthError::InternalServerError("Password verification failed".into()))?;
        if !is_valid {
            return Err(AuthError::InvalidPassword)
        }
        
        let user_public_id = user.public_id.clone();
        self.redis_repository.delete_refresh_token(&user_public_id).await?;
        
        let encoding_key = EncodingKey::from_secret(self.jwt_secret.as_bytes());
        
        let access_claims = Claims {
            iss: "editors".to_owned(),
            sub: user.public_id.clone(),
            exp: (Utc::now() + Duration::minutes(15)).timestamp() as usize,
        };
        
        let access_token = encode(
            &Header::new(Algorithm::HS512), &access_claims, &encoding_key
        )?;
        
        let refresh_claims = Claims {
            iss: "editors".to_owned(),
            sub: user.public_id.clone(),
            exp: (Utc::now() + Duration::days(1)).timestamp() as usize,
        };
        
        let refresh_token = encode(
            &Header::new(Algorithm::HS512), &refresh_claims, &encoding_key
        )?;
        
        self.redis_repository.set_refresh_token(&user_public_id, &refresh_token).await?;
        
        let cookie = Cookie::build("refresh_token", refresh_token.clone())
            .path("/")
            .secure(true)   
            .http_only(true)
            .same_site(SameSite::Lax)
            .finish();
        
        Ok(HttpResponse::Ok()
            .cookie(cookie)
            .insert_header((
                http::header::AUTHORIZATION,
                format!("Bearer {}", access_token),
                ))
            .json(json!({"message": "Login successful"}))
        )
    }
    
    pub async fn logout(&self, request: HttpRequest) -> Result<HttpResponse, AuthError> {
        let cookie = request.cookie("refresh_token").ok_or(AuthError::Unauthorized)?;
        let refresh_token = cookie.value();
        
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_bytes());
        let token_data = decode::<Claims>(
            refresh_token,
            &decoding_key,
            &Validation::new(Algorithm::HS512)
        ).map_err(|_| AuthError::Unauthorized)?;
        
        let user_public_id = &token_data.claims.sub;
        
        self.redis_repository.delete_refresh_token(user_public_id).await?;
        
        let removal_cookie = Cookie::build("refresh_token", "")
            .path("/")
            .expires(actix_web::cookie::time::OffsetDateTime::now_utc() - actix_web::cookie::time::Duration::days(1))
            .finish();
        
        Ok(HttpResponse::Ok()
            .cookie(removal_cookie)
            .json(json!({"message" : "Logout successful"}))
        )
    }
    
    pub async fn refresh_token(&self, request: HttpRequest) -> Result<HttpResponse, AuthError> {
        let cookie = request.cookie("refresh_token").ok_or(AuthError::Unauthorized)?;
        let refresh_token = cookie.value();
        
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_bytes());
        let token_data = decode::<Claims>(
            refresh_token,
            &decoding_key,
            &Validation::new(Algorithm::HS512)
        ).map_err(|_| AuthError::Unauthorized)?;
        
        let user_public_id = &token_data.claims.sub;
        let existing_refresh_token = self.redis_repository
            .get_refresh_token(user_public_id)
            .await?
            .ok_or(AuthError::Unauthorized)?;
        
        if existing_refresh_token != refresh_token {
            return Err(AuthError::Unauthorized);
        }
        
        let encoding_key = EncodingKey::from_secret(self.jwt_secret.as_bytes());
        let access_claims = Claims {
            iss: "editors".to_owned(),
            sub: user_public_id.to_string(),
            exp: (Utc::now() + Duration::minutes(15)).timestamp() as usize,
        };
        let new_access_token = encode(
            &Header::new(Algorithm::HS512), &access_claims, &encoding_key
        )?;
        
        let refresh_claims = Claims {
            iss: "editors".to_owned(),
            sub: user_public_id.to_string(),
            exp: (Utc::now() + Duration::days(1)).timestamp() as usize,
        };
        let new_refresh_token = encode(
            &Header::new(Algorithm::HS512), &refresh_claims, &encoding_key
        )?;
        
        self.redis_repository.set_refresh_token(&user_public_id, &new_refresh_token).await?;
        
        let cookie = Cookie::build("refresh_token", &new_refresh_token)
            .path("/")
            .secure(true)
            .http_only(true)
            .same_site(SameSite::Lax)
            .finish();
        
        Ok(HttpResponse::Ok()
            .cookie(cookie)
            .insert_header((
                http::header::AUTHORIZATION,
                format!("Bearer {}", new_access_token),
            ))
            .json(json!({"message": "Refresh successful"}))
        )
    }
}