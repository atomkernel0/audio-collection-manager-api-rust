use crate::{controllers::search_controller::SearchController, AppState};
use axum::{routing::get, Router};

pub struct SearchRoutes;

impl SearchRoutes {
    pub fn routes() -> Router<AppState> {
        Router::new().route("/", get(SearchController::search_albums_songs_artists))
    }
}
