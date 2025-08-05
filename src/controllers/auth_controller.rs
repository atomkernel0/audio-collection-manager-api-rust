use axum::{extract::State, http::StatusCode, Json};

use crate::{
    auth::{
        models::{LoginPayload, RegisterPayload, TokenResponse},
        token_service::TokenService,
    },
    error::Error,
    helpers::thing_helpers::{parse_id_part, thing_to_string},
    services::{auth_service::AuthService, hcaptcha_service},
    AppState, Result,
};

pub struct AuthController;

impl AuthController {
    pub async fn register_user(
        State(state): State<AppState>,
        Json(payload): Json<RegisterPayload>,
    ) -> Result<(StatusCode, Json<TokenResponse>)> {
        if !hcaptcha_service::verify_hcaptcha(&payload.hcaptcha_response).await? {
            return Err(Error::InvalidCaptcha);
        }

        let user =
            AuthService::register_user(&state.db, payload.username, payload.password).await?;

        let token = match &user.id {
            Some(id) => TokenService::create_token(
                parse_id_part(&thing_to_string(id)).to_string(),
                &state.auth_config,
            )?,
            None => return Err(Error::LoginFail),
        };

        Ok((StatusCode::CREATED, Json(TokenResponse { token })))
    }

    pub async fn login_user(
        State(state): State<AppState>,
        Json(payload): Json<LoginPayload>,
    ) -> Result<(StatusCode, Json<TokenResponse>)> {
        let token = AuthService::login_user(
            &state.db,
            &state.auth_config,
            payload.username,
            payload.password,
        )
        .await?;
        Ok((StatusCode::OK, Json(TokenResponse { token })))
    }
}
