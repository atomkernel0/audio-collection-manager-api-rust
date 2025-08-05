use crate::{
    error::Error,
    models::album::{AlbumWithArtists, AlbumWithRelations},
    services::album_service::AlbumService,
    AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};

pub struct AlbumController;

impl AlbumController {
    pub async fn get_albums(
        State(state): State<AppState>,
    ) -> Result<Json<Vec<AlbumWithArtists>>, Error> {
        let albums = AlbumService::get_albums(&state.db).await?;
        Ok(Json(albums))
    }

    pub async fn get_album(
        State(state): State<AppState>,
        Path(album_id): Path<String>,
    ) -> Result<Json<AlbumWithRelations>, Error> {
        let album = AlbumService::get_album(&state.db, &album_id)
            .await?
            .ok_or(Error::AlbumNotFound { id: album_id })?;

        Ok(Json(album))
    }
}
