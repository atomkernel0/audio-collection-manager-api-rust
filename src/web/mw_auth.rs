use crate::auth::jwt_service::{validate_token, Claims};
use crate::error::{Error, Result};
use crate::{models::user::User, AppState};
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::header;
use axum::middleware::Next;
use axum::response::Response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ctx {
    pub user_id: String,
    pub exp: usize,
    pub user: User,
}

impl Ctx {
    pub fn new(user_id: String, exp: usize, user: User) -> Self {
        Self { user_id, exp, user }
    }
}

pub async fn mw_auth(
    State(app_state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|str| str.strip_prefix("Bearer "));

    if let Some(token) = token {
        let claims: Claims = validate_token(token)?;

        let mut result = app_state
            .db
            .query(format!("SELECT * FROM {}", claims.sub))
            .await?;
        let user: Option<User> = result.take(0)?;

        if let Some(user) = user {
            let ctx = Ctx::new(claims.sub.clone(), claims.exp, user);
            req.extensions_mut().insert(ctx);
        } else {
            return Err(Error::UserNotFound {
                username: claims.sub,
            });
        }
    }
    Ok(next.run(req).await)
}
