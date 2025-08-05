use axum::{
    extract::{Path, State},
    Extension, Json,
};

use crate::{
    models::user::UserProfile, services::user_service::UserService, web::mw_auth::Ctx, AppState,
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
}
