use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::{controllers::playlist_controller::PlaylistController, AppState};

pub struct PlaylistRoutes;

impl PlaylistRoutes {
    pub fn routes() -> Router<AppState> {
        Router::new()
            .route("/create", post(PlaylistController::create_playlist_handler))
            .route(
                "/{playlist_id}/song/{song_id}",
                post(PlaylistController::add_song_to_playlist_handler),
            )
            .route(
                "/{playlist_id}/song/{song_id}",
                delete(PlaylistController::remove_song_from_playlist),
            )
            .route(
                "/user/{user_id}",
                get(PlaylistController::get_user_playlists),
            )
            .route("/user/me", get(PlaylistController::get_my_playlists))
            .route(
                "/{playlist_id}",
                get(PlaylistController::get_playlist_with_songs),
            )
            .route(
                "/{playlist_id}/like",
                post(PlaylistController::toggle_playlist_like),
            )
            .route("/", get(PlaylistController::get_public_playlists))
            .route(
                "/{playlist_id}",
                delete(PlaylistController::delete_playlist),
            )
    }
}
