use axum::{routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{web::auth, AppState, Error, Result};

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/login", post(api_login))
}

async fn api_login(payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");

    // TODO: Use DB to verify username and password
    if payload.username != "test" || payload.password != "test" {
        return Err(Error::LoginFail);
    }

    let token = auth::create_token(payload.username.to_string())?;

    let body = Json(json!({
        "result": {
            "token": token
        }
    }));

    Ok(body)
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

// TODO: Make accurate error handler
// Failed to deserialize the JSON body into the target type: missing field `username` at line 4 column 1
