use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};

use crate::{
    error::Result,
    model::album::{Album, AlbumModelController},
};

pub fn routes(mc: AlbumModelController) -> Router {
    Router::new()
        .route("/albums", get(get_all_albums))
        .route("/albums/{id}", get(get_album_by_id))
        .with_state(mc)
}

async fn get_all_albums(State(mc): State<AlbumModelController>) -> Result<Json<Vec<Album>>> {
    let albums = mc.get_all_albums().await?;
    Ok(Json(albums))
}

async fn get_album_by_id(
    State(mc): State<AlbumModelController>,
    Path(id): Path<String>,
) -> Result<Json<Album>> {
    let album = mc
        .get_album_by_id(&id)
        .await?
        .ok_or(crate::error::Error::AlbumNotFound { id })?;

    Ok(Json(album))
}
