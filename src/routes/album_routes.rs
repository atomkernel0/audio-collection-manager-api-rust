use axum::{routing::get, Router};

use crate::{controllers::album_controller::AlbumController, AppState};

pub struct AlbumRoutes;

impl AlbumRoutes {
    pub fn routes() -> Router<AppState> {
        Router::new()
            .route("/", get(AlbumController::get_albums))
            .route("/{album_id}", get(AlbumController::get_album))
    }
}
