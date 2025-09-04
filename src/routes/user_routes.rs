use axum::{routing::{get, patch, delete}, Router};

use crate::{controllers::user_controller::UserController, AppState};

pub struct UserRoutes;

impl UserRoutes {
    pub fn routes() -> Router<AppState> {
        Router::new()
            .route("/me", get(UserController::get_my_profile))
            .route("/{user_id}", get(UserController::get_user_profile))
            .route("/check-username", get(UserController::check_username))

            .route("/me", patch(UserController::update_my_username))
            .route("/me/password", patch(UserController::change_my_password))

            .route("/me", delete(UserController::delete_my_account))
    }
}
