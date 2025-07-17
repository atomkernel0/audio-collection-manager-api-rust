use axum::{routing::post, Router};

use crate::{controllers::playlist_controller::PlaylistController, AppState};

pub struct PlaylistRoutes;

impl PlaylistRoutes {
    pub fn routes() -> Router<AppState> {
        Router::new()
            .route("/create", post(PlaylistController::create_playlist_handler))
            .route(
                "/{playlist_id}/add/{song_id}",
                post(PlaylistController::add_song_to_playlist_handler),
            )
    }
}
