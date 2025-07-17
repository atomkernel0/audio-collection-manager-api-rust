use crate::error::{Error, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user_id)
    pub exp: usize,  // Expiration time
    pub iat: usize,  // Issued at
}

pub fn create_token(user_id: &str) -> Result<String> {
    let secret = env::var("JWT_SECRET")
        .map_err(|_| Error::EnvVarError("JWT_SECRET not found".to_string()))?;

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::hours(24)).timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| Error::TokenCreationError)
}

pub fn validate_token(token: &str) -> Result<Claims> {
    let secret = env::var("JWT_SECRET")
        .map_err(|_| Error::EnvVarError("JWT_SECRET not found".to_string()))?;

    let decoding_key = jsonwebtoken::DecodingKey::from_secret(secret.as_ref());

    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);

    jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|_| Error::InvalidToken)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_token() {
        env::set_var("JWT_SECRET", "test-secret");
        let user_id = "user123";
        let token = create_token(user_id).unwrap();
        assert!(!token.is_empty());

        // You could add more robust validation here if needed,
        // like decoding the token and checking the claims.
    }
}
