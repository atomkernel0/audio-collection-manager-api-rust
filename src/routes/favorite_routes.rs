use crate::{controllers::favorite_controller::FavoriteController, AppState};
use axum::{
    routing::{get, post},
    Router,
};

pub struct FavoriteRoutes;

impl FavoriteRoutes {
    pub fn routes() -> Router<AppState> {
        Router::new()
            .route("/albums", get(FavoriteController::get_favorite_albums))
            .route(
                "/albums/ids",
                get(FavoriteController::get_favorite_album_ids),
            )
            .route("/artists", get(FavoriteController::get_favorite_artists))
            .route(
                "/artists/ids",
                get(FavoriteController::get_favorite_artist_ids),
            )
            .route("/songs", get(FavoriteController::get_favorite_songs))
            .route("/songs/ids", get(FavoriteController::get_favorite_song_ids))
            .route(
                "/statistics",
                get(FavoriteController::get_favorites_statistics),
            )
            .route(
                "/albums/{album_id}/toggle",
                post(FavoriteController::toggle_favorite_album),
            )
            .route(
                "/songs/{song_id}/toggle",
                post(FavoriteController::toggle_favorite_song),
            )
            .route(
                "/artists/{artist_id}/toggle",
                post(FavoriteController::toggle_favorite_artist),
            )
            .route(
                "/albums/{album_id}/check",
                get(FavoriteController::check_favorite_album),
            )
            .route(
                "/artists/{artist_id}/check",
                get(FavoriteController::check_favorite_artist),
            )
            .route(
                "/songs/{song_id}/check",
                get(FavoriteController::check_favorite_song),
            )
    }
}
