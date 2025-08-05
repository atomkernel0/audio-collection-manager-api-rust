use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::{ConnectInfo, MatchedPath, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::{web::mw_auth::Ctx, AppState};

pub async fn rate_limit_middleware(
    State(app_state): State<AppState>,
    ConnectInfo(ip): ConnectInfo<SocketAddr>,
    matched_path: MatchedPath,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = matched_path.as_str();
    let song_id = if let Some(id) = path.split('/').nth(2) {
        id
    } else {
        // This case should ideally not be reached if routes are set up correctly.
        // Returning a server error because it indicates a configuration issue.
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let key = req
        .extensions()
        .get::<Ctx>()
        .map(|ctx| format!("{}:{}", ctx.user_id, song_id))
        .unwrap_or_else(|| format!("{}:{}", ip, song_id));

    if app_state.rate_limit_cache.get(&key).await.is_some() {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    app_state.rate_limit_cache.insert(key, ()).await;

    Ok(next.run(req).await)
}
