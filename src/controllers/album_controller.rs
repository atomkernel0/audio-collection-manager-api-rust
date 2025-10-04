use crate::{
    error::Error,
    models::album::{AlbumWithArtists, AlbumWithRelations, AlbumsMetaResponse},
    models::database_helpers::CountResult,
    services::album_service::AlbumService,
    middlewares::mw_auth::Ctx,
    AppState,
    Result,
};
use axum::{
    extract::{Path, State, Query},
    Extension,
    Json,
};
use serde::Deserialize;

pub struct AlbumController;

#[derive(Debug, Deserialize)]
pub struct InitialAlbumsQuery {
    #[serde(default = "default_limit")]
    pub limit: u32,
}
fn default_limit() -> u32 {
    30
}

#[derive(Debug, Deserialize)]
pub struct AlbumsBatchQuery {
    #[serde(default)]
    pub offset: u32,
    #[serde(default = "default_batch_limit")]
    pub limit: u32,
}
fn default_batch_limit() -> u32 {
    50
}

#[derive(Debug, Deserialize)]
pub struct AlbumsFilterQuery {
    #[serde(default)]
    pub offset: u32,
    #[serde(default = "default_batch_limit")]
    pub limit: u32,
    pub genres: Option<String>, // Comma-separated genres
    pub search: Option<String>,
    pub sort_by: Option<String>, // "popular", "recent", "alphabetical"
}

impl AlbumController {
    pub async fn get_albums(
        State(state): State<AppState>,
    ) -> Result<Json<Vec<AlbumWithArtists>>> {
        let albums = AlbumService::get_albums(&state.db).await?;
        Ok(Json(albums))
    }

    pub async fn get_album(
        State(state): State<AppState>,
        Path(album_id): Path<String>,
    ) -> Result<Json<AlbumWithRelations>> {
        let album = AlbumService::get_album(&state.db, &album_id)
            .await?
            .ok_or(Error::AlbumNotFound { id: album_id })?;

        Ok(Json(album))
    }

    pub async fn get_initial_albums_with_meta(
        State(state): State<AppState>,
        Query(params): Query<InitialAlbumsQuery>,
    ) -> Result<Json<AlbumsMetaResponse>> {
        let response =
            AlbumService::get_initial_albums_with_meta(&state.db, params.limit).await?;
        Ok(Json(response))
    }

    pub async fn get_albums_batch(
        State(state): State<AppState>,
        Query(params): Query<AlbumsBatchQuery>,
    ) -> Result<Json<Vec<AlbumWithArtists>>> {
        let albums =
            AlbumService::get_albums_batch(&state.db, params.offset, params.limit).await?;
        Ok(Json(albums))
    }

    pub async fn get_albums_filtered(
        State(state): State<AppState>,
        Query(params): Query<AlbumsFilterQuery>,
    ) -> Result<Json<AlbumsMetaResponse>> {
        // Parser les genres si fournis
        let genres = params.genres.as_ref().map(|g| {
            g.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<String>>()
        });

        let response = AlbumService::get_albums_filtered(
            &state.db,
            params.offset,
            params.limit,
            genres,
            params.search,
            params.sort_by,
        )
        .await?;

        Ok(Json(response))
    }

    pub async fn get_albums_count(
        State(state): State<AppState>,
    ) -> Result<Json<serde_json::Value>> {
        let mut response = state
            .db
            .query("SELECT count() AS total FROM album GROUP ALL;")
            .await?;
        let count: Option<CountResult> = response.take(0)?;
        let total_count = count.map(|c| c.total as u32).unwrap_or(0);

        Ok(Json(serde_json::json!({ "total_count": total_count })))
    }

    pub async fn listen_to_album(
        State(state): State<AppState>,
        Path(album_id): Path<String>,
        ctx: Option<Extension<Ctx>>,
    ) -> Result<Json<bool>> {
        let user_id = ctx.as_ref().map(|c| c.user_id.as_str());

        let success = AlbumService::listen_to_album(&state.db, &album_id, user_id).await?;

        Ok(Json(success))
    }
}
