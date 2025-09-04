use crate::{controllers::song_controller::SongController, AppState};
use axum::{
    routing::{get, post},
    Router,
};

pub struct SongRoutes;

impl SongRoutes {
    pub fn routes() -> Router<AppState> {
        Router::new()
            .route(
                "/{song_id}/listen",
                post(SongController::listen_to_song),
            )
            .route("/{song_id}/album", get(SongController::get_album_from_song))
            .route("/recents", get(SongController::get_user_recent_listens))
    }
}
