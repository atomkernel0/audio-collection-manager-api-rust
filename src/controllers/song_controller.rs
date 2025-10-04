use crate::{
    error::Result,
    models::{
        album::AlbumWithRelations,
        pagination::{PaginatedResponse, PaginationQuery},
        song::SongWithRelations,
    },
    services::song_service::{SongService, ListenResult},
    middlewares::mw_auth::Ctx,
    AppState, Error,
};
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};

pub struct SongController;

impl SongController {
    pub async fn listen_to_song(
        State(state): State<AppState>,
        Path(song_id): Path<String>,
        ctx: Option<Extension<Ctx>>,
    ) -> Result<Json<ListenResult>> {
        let user_id = ctx.as_ref().map(|c| c.user_id.as_str());

        let result = SongService::listen_to_song(&state.db, &song_id, user_id).await?;

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
