use axum::{routing::post, Router};

use crate::controllers::auth_controller::{login_handler, register_handler};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register_handler))
        .route("/auth/login", post(login_handler))
}
