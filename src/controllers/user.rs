use crate::{
    services::user_service,
    web::mw_auth::Ctx,
    {AppState, Result},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};

/*
pub async fn like_song_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
    Path(song_id): Path<String>,
) -> Result<(StatusCode, Json<Value>)> {
    let liked = user::like_song(&state.db, &ctx.user_id, &song_id).await?;
    let status_code = if liked {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status_code, Json(json!({ "liked": liked }))))
}
*/
