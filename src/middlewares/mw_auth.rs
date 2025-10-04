use crate::auth::token_service::{Claims, TokenService};
use crate::error::{Error, Result};
use crate::helpers::thing_helpers::{create_user_thing, parse_id_part};
use crate::{models::user::UserRecord, AppState};
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
    pub user: UserRecord,
}

impl Ctx {
    pub fn new(user_id: String, exp: usize, user: UserRecord) -> Self {
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
        .and_then(|str| str.strip_prefix("Bearer "))
        .ok_or(Error::AuthFailNoAuthTokenCookie)?;

    let claims: Claims = TokenService::validate_token(token, &app_state.auth_config)?;

    let sub_str = claims.sub.clone();
    let user_id = parse_id_part(&sub_str);
    let user_thing = create_user_thing(user_id);

    let mut result = app_state
        .db
        .query("SELECT * FROM $user_thing")
        .bind(("user_thing", user_thing))
        .await?;
    let user: Option<UserRecord> = result.take(0)?;

    let user = user.ok_or(Error::UserNotFound {
        username: claims.sub.clone(),
    })?;

    let ctx = Ctx::new(claims.sub, claims.exp as usize, user);
    req.extensions_mut().insert(ctx);

    Ok(next.run(req).await)
}
