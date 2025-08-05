use crate::{controllers::auth_controller::AuthController, AppState};
use axum::{routing::post, Router};

pub struct AuthRoutes;

impl AuthRoutes {
    pub fn routes() -> Router<AppState> {
        Router::new()
            .route("/register", post(AuthController::register_user))
            .route("/login", post(AuthController::login_user))
    }
}
