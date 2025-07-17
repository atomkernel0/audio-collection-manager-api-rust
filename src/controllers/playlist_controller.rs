use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};

use surrealdb::sql::Thing;

use crate::{
    models::playlist::CreatePlaylistRequest, services::playlist_service::PlaylistService,
    web::mw_auth::Ctx, AppState, Error,
};

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
    ) -> Result<(), Error> {
        let result =
            PlaylistService::add_song_to_playlist(&state.db, &ctx.user_id, &song_id, &playlist_id)
                .await?;

        Ok(result)
    }
}
