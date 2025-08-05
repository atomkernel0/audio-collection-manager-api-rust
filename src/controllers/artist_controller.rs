use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    models::artist::{Artist, ArtistWithAlbumsAndTopSongs},
    services::artist_service::ArtistService,
    AppState, Error,
};

pub struct ArtistController;

impl ArtistController {
    pub async fn get_artists(State(state): State<AppState>) -> Result<Json<Vec<Artist>>, Error> {
        let albums = ArtistService::get_artists(&state.db).await?;
        Ok(Json(albums))
    }

    pub async fn get_artist(
        State(state): State<AppState>,
        Path(artist_id): Path<String>,
    ) -> Result<Json<ArtistWithAlbumsAndTopSongs>, Error> {
        let album = ArtistService::get_artist(&state.db, &artist_id)
            .await?
            .ok_or(Error::ArtistNotFound { id: artist_id })?;

        Ok(Json(album))
    }
}
