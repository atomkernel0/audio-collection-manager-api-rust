use crate::{
    error::Result, middlewares::mw_auth::Ctx, models::{
        album::AlbumWithRelations,
        pagination::{PaginatedResponse, PaginationQuery},
        song::{SongWithRelations},
    }, services::song_service::{ListenResult, SongService}, validators::listen_validator::{ListenValidator, ValidationResult}, AppState, Error
};
use axum::{
    extract::{ConnectInfo, Path, Query, State},
    Extension, Json,
};
use std::net::SocketAddr;

pub struct SongController;

impl SongController {
    pub async fn listen_to_song(
        State(state): State<AppState>,
        Path(song_id): Path<String>,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        ctx: Option<Extension<Ctx>>,
    ) -> Result<Json<ListenResult>> {
        let user_id = ctx.as_ref().map(|c| c.user_id.as_str());
        let client_ip = addr.ip().to_string();

        let song = SongService::get_song_by_id(&state.db, &song_id)
            .await?
            .ok_or_else(|| Error::SongNotFound {
                id: song_id.clone(),
            })?;

        let validation_result =
            ListenValidator::validate_listen(&state.db, &song_id, user_id, Some(&client_ip), song.duration.as_secs())
                .await?;

        if let ValidationResult::RateLimited {
            reason,
            retry_after_secs,
        } = validation_result
        {
            return Err(Error::RateLimited {
                reason,
                retry_after_secs,
            });
        }

        let result =
            SongService::listen_to_song(&state.db, &song_id, user_id, song.duration)
                .await?;

        Ok(Json(result))
    }

    pub async fn get_user_recent_listens(
        State(state): State<AppState>,
        Extension(ctx): Extension<Ctx>,
        Query(query): Query<PaginationQuery>,
    ) -> Result<Json<PaginatedResponse<SongWithRelations>>> {
        let result = SongService::get_user_recent_listens(&state.db, &ctx.user_id, &query).await?;

        Ok(Json(result))
    }

    pub async fn get_album_from_song(
        State(state): State<AppState>,
        Path(song_id): Path<String>,
    ) -> Result<Json<AlbumWithRelations>> {
        let album = SongService::get_album_from_song(&state.db, &song_id)
            .await?
            .ok_or(Error::AlbumNotFound { id: song_id })?;

        Ok(Json(album))
    }
}
