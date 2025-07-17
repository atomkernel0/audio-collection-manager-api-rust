use crate::{controllers::user, web::mw_auth::mw_auth, AppState};
use axum::{middleware, routing::post, Router};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
}
