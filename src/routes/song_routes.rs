use crate::{
    controllers::{self, song_handler::listen_song_handler},
    models::album::AlbumWithRelations,
    AppState,
};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use crate::error::{Error, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/songs/{id}/listen", post(listen_song_handler))
        .route("/songs/{id}/album", get(get_album_from_song_handler))
}

async fn get_album_from_song_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AlbumWithRelations>> {
    let album = controllers::song_handler::get_album_from_song(&state.db, &id)
        .await?
        .ok_or(Error::AlbumNotFound { id })?;

    Ok(Json(album))
}
