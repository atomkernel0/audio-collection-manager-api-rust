use crate::{error::Error, middlewares::mw_auth::Ctx, AppState};
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use reqwest::StatusCode;

use crate::{models::favorite::*, services::favorite_service::FavoriteService};

pub struct FavoriteController;

impl FavoriteController {
    pub async fn toggle_favorite_album(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Path(album_id): Path<String>,
    ) -> Result<(StatusCode, Json<bool>), Error> {
        let liked =
            FavoriteService::toggle_favorite_album(&state.db, &ctx.user_id, &album_id).await?;

        let status_code = if liked {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        };

        Ok((status_code, Json(liked)))
    }

    pub async fn toggle_favorite_song(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Path(song_id): Path<String>,
    ) -> Result<(StatusCode, Json<bool>), Error> {
        let liked =
            FavoriteService::toggle_favorite_song(&state.db, &ctx.user_id, &song_id).await?;

        let status_code = if liked {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        };

        Ok((status_code, Json(liked)))
    }

    pub async fn toggle_favorite_artist(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Path(artist_id): Path<String>,
    ) -> Result<(StatusCode, Json<bool>), Error> {
        let liked =
            FavoriteService::toggle_favorite_artist(&state.db, &ctx.user_id, &artist_id).await?;

        let status_code = if liked {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        };

        Ok((status_code, Json(liked)))
    }

    pub async fn get_favorite_albums(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Query(query): Query<FavoritesQuery>,
    ) -> Result<Json<FavoritesResponse<AlbumWithFavoriteMetadata>>, Error> {
        let albums = FavoriteService::get_favorite_albums(&state.db, &ctx.user_id, &query).await?;

        Ok(Json(albums))
    }

    pub async fn get_favorite_album_ids(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
    ) -> Result<Json<Vec<String>>, Error> {
        let album_ids = FavoriteService::get_favorite_album_ids(&state.db, &ctx.user_id).await?;

        Ok(Json(album_ids))
    }

    pub async fn get_favorite_artists(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Query(query): Query<FavoritesQuery>,
    ) -> Result<Json<FavoritesResponse<ArtistWithFavoriteMetadata>>, Error> {
        let artists =
            FavoriteService::get_favorite_artists(&state.db, &ctx.user_id, &query).await?;

        Ok(Json(artists))
    }

    pub async fn get_favorite_artist_ids(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
    ) -> Result<Json<Vec<String>>, Error> {
        let artist_ids = FavoriteService::get_favorite_artist_ids(&state.db, &ctx.user_id).await?;

        Ok(Json(artist_ids))
    }

    pub async fn get_favorite_songs(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Query(query): Query<FavoritesQuery>,
    ) -> Result<Json<FavoritesResponse<SongWithFavoriteMetadata>>, Error> {
        let songs = FavoriteService::get_favorite_songs(&state.db, &ctx.user_id, &query).await?;

        Ok(Json(songs))
    }

    pub async fn get_favorite_song_ids(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
    ) -> Result<Json<Vec<String>>, Error> {
        let song_ids = FavoriteService::get_favorite_song_ids(&state.db, &ctx.user_id).await?;

        Ok(Json(song_ids))
    }

    pub async fn check_favorite_album(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Path(album_id): Path<String>,
    ) -> Result<Json<bool>, Error> {
        let is_favorite =
            FavoriteService::check_favorite_album(&state.db, &ctx.user_id, &album_id).await?;

        Ok(Json(is_favorite))
    }

    pub async fn check_favorite_artist(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Path(artist_id): Path<String>,
    ) -> Result<Json<bool>, Error> {
        let is_favorite =
            FavoriteService::check_favorite_artist(&state.db, &ctx.user_id, &artist_id).await?;

        Ok(Json(is_favorite))
    }

    pub async fn check_favorite_song(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Path(song_id): Path<String>,
    ) -> Result<Json<bool>, Error> {
        let is_favorite =
            FavoriteService::check_favorite_song(&state.db, &ctx.user_id, &song_id).await?;

        Ok(Json(is_favorite))
    }

    pub async fn get_favorites_statistics(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
    ) -> Result<Json<FavoritesStatistics>, Error> {
        let statistics = FavoriteService::get_statistics(&state.db, &ctx.user_id).await?;

        Ok(Json(statistics))
    }
}
