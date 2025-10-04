use axum::{
    extract::{Path, State, Query},
    Extension, Json,
};

use crate::{
    models::user::UserProfile, services::user_service::UserService, middlewares::mw_auth::Ctx, AppState,
    Error,
};

pub struct UserController;

impl UserController {
    pub async fn get_my_profile(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
    ) -> Result<Json<UserProfile>, Error> {
        let result = UserService::get_user_profile(&state.db, &ctx.user_id).await?;
        Ok(Json(result))
    }

    pub async fn get_user_profile(
        State(state): State<AppState>,
        // todo: forcer l'auth
        Path(user_id): Path<String>,
    ) -> Result<Json<UserProfile>, Error> {
        let result = UserService::get_user_profile(&state.db, &user_id).await?;
        Ok(Json(result))
    }

    pub async fn update_my_username(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Json(payload): Json<UpdateUsernamePayload>,
    ) -> Result<Json<UserProfile>, Error> {
        let updated = UserService::update_username(&state.db, &ctx.user_id, &payload.username).await?;
        Ok(Json(updated))
    }

    pub async fn check_username(
        State(state): State<AppState>,
        Query(params): Query<CheckUsernameQuery>,
    ) -> Result<Json<AvailabilityResponse>, Error> {
        let available = UserService::check_if_username_is_available(&state.db, &params.username).await?;
        Ok(Json(AvailabilityResponse { available }))
    }

    pub async fn delete_my_account(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
    ) -> Result<Json<DeleteAccountResponse>, Error> {
        let ok = UserService::delete_user(&state.db, &ctx.user_id).await?;
        Ok(Json(DeleteAccountResponse { success: ok, message: None }))
    }

    pub async fn change_my_password(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Json(payload): Json<ChangePasswordPayload>,
    ) -> Result<Json<bool>, Error> {
        let ok = UserService::change_password(
            &state.db,
            &ctx.user_id,
            &payload.current_password,
            &payload.new_password,
        ).await?;
        Ok(Json(ok))
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateUsernamePayload {
    pub username: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct CheckUsernameQuery {
    pub username: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ChangePasswordPayload {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, serde::Serialize)]
pub struct AvailabilityResponse {
    pub available: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct DeleteAccountResponse {
    pub success: bool,
    pub message: Option<String>,
}
