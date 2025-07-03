use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Error, Result};

const JWT_SECRET: &[u8] = b"secret"; //TODO: Implement secret from envvar
const TOKEN_DURATION_MIN: i64 = 60;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // subject (user_id)
    pub exp: i64,    // expiration time
    pub iat: i64,    // issued at
    pub nbf: i64,    // not before
    pub iss: String, // issuer
    pub aud: String, // audience
    pub jti: String, // jtw id
}

// Implement `new` for Claims
impl Claims {
    // TODO - Make this part of the user session system.
    //      - Need to create a Web Session Data struct with
    //        user info and session info, which will be the Claims
    pub fn new(sub: String) -> Self {
        let iat = Utc::now();
        let exp = iat + Duration::minutes(TOKEN_DURATION_MIN);

        Self {
            sub,
            exp: exp.timestamp(),
            iat: iat.timestamp(),
            nbf: iat.timestamp(),
            iss: "https://www.rust-lang.org".to_string(), // TODO: needs to be updated.
            aud: "https://www.rust-lang.org".to_string(), // TODO: needs to be updated.
            jti: Uuid::new_v4().to_string(),
        }
    }
}

pub fn create_token(sub: String) -> Result<String> {
    let claims = Claims::new(sub);
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .map_err(|e| {
        eprintln!("->> JWT - create_token - error: {e:?}");
        Error::TokenCreationError
    })?;
    Ok(token)
}

pub fn validate_token(token: &str) -> Result<Claims> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.set_audience(&["https://www.rust-lang.org"]); // TODO: Needs to be updated
    validation.set_issuer(&["https://www.rust-lang.org"]); // TODO: Needs to be updated

    let decoded = decode::<Claims>(token, &DecodingKey::from_secret(JWT_SECRET), &validation)
        .map_err(|e| {
            eprintln!("->> JWT - validate_token - error: {e:?}");
            Error::InvalidToken
        })?;
    Ok(decoded.claims)
}
