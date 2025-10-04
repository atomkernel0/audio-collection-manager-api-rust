use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::Serialize;

use surrealdb::sql::Thing;

use crate::{
    models::playlist::{CreatePlaylistRequest, Playlist, PlaylistWithSongs},
    services::playlist_service::PlaylistService,
    middlewares::mw_auth::Ctx,
    AppState, Error,
};

#[derive(Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

pub struct PlaylistController;

impl PlaylistController {
    pub async fn create_playlist_handler(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Json(payload): Json<CreatePlaylistRequest>,
    ) -> Result<Json<Thing>, Error> {
        let created_playlist_thing =
            PlaylistService::create_playlist(&state.db, &ctx.user_id, payload).await?;

        Ok(Json(created_playlist_thing))
    }

    pub async fn add_song_to_playlist_handler(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Path((playlist_id, song_id)): Path<(String, String)>,
    ) -> Result<Json<SuccessResponse>, Error> {
        let _ =
            PlaylistService::add_song_to_playlist(&state.db, &ctx.user_id, &song_id, &playlist_id)
                .await?;

        Ok(Json(SuccessResponse { success: true }))
    }

    pub async fn remove_song_from_playlist(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Path((playlist_id, song_id)): Path<(String, String)>,
    ) -> Result<Json<SuccessResponse>, Error> {
        let _ = PlaylistService::remove_song_from_playlist(
            &state.db,
            &ctx.user_id,
            &song_id,
            &playlist_id,
        )
        .await?;

        Ok(Json(SuccessResponse { success: true }))
    }

    /// Récupère les playlists de l'utilisateur connecté
    pub async fn get_my_playlists(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
    ) -> Result<Json<Vec<Playlist>>, Error> {
        let result = PlaylistService::get_user_playlists(&state.db, &ctx.user_id).await?;
        Ok(Json(result))
    }

    /// Récupère les playlists d'un utilisateur spécifique
    pub async fn get_user_playlists(
        State(state): State<AppState>,
        //        Extension(ctx): Extension<Ctx>, //todo : verifier si ça "force" la connexion car il faut authentification pour get des données comme ça sur le site
        Path(user_id): Path<String>,
    ) -> Result<Json<Vec<Playlist>>, Error> {
        if user_id.trim().is_empty() {
            return Err(Error::InvalidInput {
                reason: "L'ID utilisateur ne peut pas être vide".to_string(),
            });
        }

        let result = PlaylistService::get_user_playlists(&state.db, &user_id).await?;

        Ok(Json(result))
    }

    pub async fn get_playlist_with_songs(
        State(state): State<AppState>,
        Path(playlist_id): Path<String>,
    ) -> Result<Json<PlaylistWithSongs>, Error> {
        let result = PlaylistService::get_playlist_with_songs(&state.db, &playlist_id).await?;

        Ok(Json(result))
    }

    pub async fn toggle_playlist_like(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Path(playlist_id): Path<String>,
    ) -> Result<Json<bool>, Error> {
        let result =
            PlaylistService::toggle_playlist_like(&state.db, &ctx.user_id, &playlist_id).await?;

        Ok(Json(result))
    }

    pub async fn get_public_playlists(
        State(state): State<AppState>,
    ) -> Result<Json<Vec<Playlist>>, Error> {
        let result = PlaylistService::get_public_playlists(&state.db).await?;

        Ok(Json(result))
    }

    pub async fn delete_playlist(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Path(playlist_id): Path<String>,
    ) -> Result<Json<SuccessResponse>, Error> {
        let _ = PlaylistService::delete_playlist(&state.db, &ctx.user_id, &playlist_id).await?;

        Ok(Json(SuccessResponse { success: true }))
    }
}
