use axum::{extract::State, http::StatusCode, Json};

use crate::{
    auth::models::{LoginPayload, RegisterPayload, TokenResponse},
    error::Error,
    services::{auth_service, hcaptcha_service},
    AppState, Result,
};

pub async fn register_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<(StatusCode, Json<TokenResponse>)> {
    if !hcaptcha_service::verify_hcaptcha(&payload.hcaptcha_response).await? {
        return Err(Error::InvalidCaptcha);
    }

    let user = auth_service::register_user(&state.db, payload.username, payload.password).await?;
    let token = crate::auth::jwt_service::create_token(&user.id.unwrap().to_string())?;

    Ok((StatusCode::CREATED, Json(TokenResponse { token })))
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<(StatusCode, Json<TokenResponse>)> {
    let token = auth_service::login_user(&state.db, payload.username, payload.password).await?;
    Ok((StatusCode::OK, Json(TokenResponse { token })))
}
