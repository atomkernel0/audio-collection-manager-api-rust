use crate::{
    error::Result, models::album::AlbumWithRelations, services::song_service, web::mw_auth::Ctx,
    AppState,
};
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use surrealdb::{engine::any::Any, Surreal};

pub async fn listen_song_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ctx: Option<Extension<Ctx>>,
) -> Result<Json<bool>> {
    let user_id = ctx.as_ref().map(|c| c.user_id.as_str());

    let success = song_service::listen_to_song(&state.db, &id, user_id).await?;

    Ok(Json(success))
}

pub async fn get_album_from_song(
    db: &Surreal<Any>,
    song_id: &str,
) -> Result<Option<AlbumWithRelations>> {
    let album = song_service::get_album_from_song(db, song_id).await?;

    Ok(album)
}
