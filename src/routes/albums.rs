use crate::{
    controllers::{self},
    error::{Error, Result},
    models::album::{AlbumWithArtists, AlbumWithRelations},
    AppState,
};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use surrealdb::sql::Thing;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_albums_handler))
        .route("/{id}", get(get_album_handler))
}

async fn get_album_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AlbumWithRelations>> {
    let thing_id = Thing::from(("album", id.as_str()));

    let album = controllers::album::get_album(&state.db, thing_id)
        .await?
        .ok_or(Error::AlbumNotFound { id })?;

    Ok(Json(album))
}

async fn get_albums_handler(State(state): State<AppState>) -> Result<Json<Vec<AlbumWithArtists>>> {
    let albums = controllers::album::get_albums(&state.db).await?;
    Ok(Json(albums))
}
