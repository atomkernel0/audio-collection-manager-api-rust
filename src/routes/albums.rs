use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};

use crate::{
    controllers,
    error::Result,
    models::albums::{Album, GetAllAlbumsResponse},
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/albums", get(get_all_albums_handler))
    // .route("/albums/{id}", get(get_album_by_id_handler))
}

async fn get_all_albums_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<GetAllAlbumsResponse>>> {
    let albums = controllers::albums::get_all_albums(&state.db).await?;
    Ok(Json(albums))
}

// async fn get_album_by_id_handler(
//     State(state): State<AppState>,
//     Path(id): Path<String>,
// ) -> Result<Json<Album>> {
//     let album = controllers::albums::get_album_by_id(&state.db, &id)
//         .await?
//         .ok_or(crate::error::Error::AlbumNotFound { id })?;
//     Ok(Json(album))
// }
