use axum::{routing::get, Router};

use crate::{controllers::artist_controller::ArtistController, AppState};

pub struct ArtistRoutes;

impl ArtistRoutes {
    pub fn routes() -> Router<AppState> {
        Router::new()
            .route("/", get(ArtistController::get_artists))
            .route("/{artist_id}", get(ArtistController::get_artist))
    }
}
