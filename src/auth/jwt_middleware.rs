use std::future::{ready, Ready};
use actix_web::{FromRequest, HttpRequest};
use actix_web::dev::Payload;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use crate::auth::auth_error::AuthError;
use crate::auth::auth_model::{AuthenticatedUser, Claims};
use crate::config;

impl FromRequest for AuthenticatedUser {
    type Error = AuthError;
    type Future = Ready<Result<Self, Self::Error>>;
    
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let config = config::Config::from_env();
        let jwt_secret = config.jwt_secret.clone();
        
        let token = match req.headers().get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|str_value| {
                if str_value.starts_with("Bearer ") {
                    Some(str_value[7..].to_string())
                } else {
                    None
                }
            })
        {
            Some(token) => token,
            None => return ready(Err(AuthError::Unauthorized)),
        };
        
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
        match decode::<Claims>(&token, &decoding_key, &Validation::new(Algorithm::HS512)) {
            Ok(token_data) => {
                ready(Ok(AuthenticatedUser {
                    public_id: token_data.claims.sub,
                }))
            }
            Err(_) => ready(Err(AuthError::Unauthorized)),
        }
    }
}