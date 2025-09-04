use axum::{
    routing::{get, post},
    Router,
};

use crate::{controllers::album_controller::AlbumController, AppState};

pub struct AlbumRoutes;

impl AlbumRoutes {
    pub fn routes() -> Router<AppState> {
        Router::new()
            .route("/", get(AlbumController::get_albums))
            .route("/{album_id}", get(AlbumController::get_album))
            .route("/{album_id}/listen", post(AlbumController::listen_to_album))
            .route(
                "/initial",
                get(AlbumController::get_initial_albums_with_meta),
            )
            .route("/batch", get(AlbumController::get_albums_batch))
            .route("/filtered", get(AlbumController::get_albums_filtered))
            .route("/count", get(AlbumController::get_albums_count))
    }
}
