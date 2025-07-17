use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};

use crate::{
    controllers,
    error::Result,
    models::artist::{Artist, ArtistWithAlbumsAndTopSongs},
    AppState, Error,
};

use surrealdb::sql::Thing;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/artists", get(get_artists_handler))
        .route("/artists/{id}", get(get_artist_handler))
}

async fn get_artists_handler(State(state): State<AppState>) -> Result<Json<Vec<Artist>>> {
    let artists = controllers::artist::get_artists(&state.db).await?;
    Ok(Json(artists))
}

async fn get_artist_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ArtistWithAlbumsAndTopSongs>> {
    let thing_id = Thing::from(("artist", id.as_str()));

    let artist = controllers::artist::get_artist(&state.db, thing_id)
        .await?
        .ok_or(Error::ArtistNotFound { id })?;

    Ok(Json(artist))
}
