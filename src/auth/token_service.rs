use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

use crate::Result;

#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub website_url: String,
    pub token_duration_min: i64,
    pub jwt_algorithm: Algorithm,
}

impl AuthConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            jwt_secret: env::var("JWT_SECRET")?,
            website_url: env::var("WEBSITE_URL")?,
            token_duration_min: env::var("TOKEN_DURATION_MIN")
                .unwrap_or_else(|_| "60".to_string())
                .parse::<i64>()
                .unwrap_or(60),
            jwt_algorithm: Algorithm::HS256,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // subject (user_id)
    pub exp: usize,  // expiration time
    pub iat: usize,  // issued at
    pub nbf: usize,  // not before
    pub iss: String, // issuer
    pub aud: String, // audience
    pub jti: String, // jtw id
}

impl Claims {
    pub fn new(sub: String, config: &AuthConfig) -> Self {
        let iat = Utc::now();
        let exp = iat + Duration::minutes(config.token_duration_min);

        Self {
            sub,
            exp: exp.timestamp() as usize,
            iat: iat.timestamp() as usize,
            nbf: iat.timestamp() as usize,
            iss: config.website_url.to_string(),
            aud: config.website_url.to_string(),
            jti: Uuid::new_v4().to_string(),
        }
    }
}

pub struct TokenService;

impl TokenService {
    pub fn create_token(sub: String, config: &AuthConfig) -> Result<String> {
        let claims = Claims::new(sub, config);
        let token = encode(
            &Header::new(config.jwt_algorithm),
            &claims,
            &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
        )?;
        Ok(token)
    }

    pub fn validate_token(token: &str, config: &AuthConfig) -> Result<Claims> {
        let mut validation = Validation::new(config.jwt_algorithm);
        validation.set_audience(&[config.website_url.to_string()]);
        validation.set_issuer(&[config.website_url.to_string()]);

        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            &validation,
        )?;
        Ok(decoded.claims)
    }
}
