use axum::{routing::get, Router};

use crate::{controllers::user_controller::UserController, AppState};

pub struct UserRoutes;

impl UserRoutes {
    pub fn routes() -> Router<AppState> {
        Router::new()
            .route("/me", get(UserController::get_my_profile))
            .route("/{user_id}", get(UserController::get_user_profile))
    }
}
