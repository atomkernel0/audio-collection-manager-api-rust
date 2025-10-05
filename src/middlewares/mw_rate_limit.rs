use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::{middlewares::mw_auth::Ctx, AppState};

/// Global rate limiting middleware that only blocks heavy spammers
///
/// Limits:
/// - 150 requests per minute per user/IP (very generous for normal usage)
/// - 10 requests per 10 seconds (prevents rapid bursts)
///
/// This allows users to listen to multiple songs quickly without issues
/// while still protecting against abuse.
///
/// Uses a simple counter-based approach with the moka cache which has TTL support.
pub async fn rate_limit_middleware(
    State(app_state): State<AppState>,
    ConnectInfo(ip): ConnectInfo<SocketAddr>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Identify the requester (user or IP)
    let identifier = req
        .extensions()
        .get::<Ctx>()
        .map(|ctx| format!("user:{}", ctx.user_id))
        .unwrap_or_else(|| format!("ip:{}", ip.ip()));

    // Generate unique request ID for this request
    let request_id = uuid::Uuid::new_v4();
    
    // Minute-based rate limit (150 requests per minute)
    let minute_key = format!("rl:min:{}:{}", identifier, request_id);
    app_state.rate_limit_cache.insert(minute_key.clone(), ()).await;
    
    // Count requests in the last minute
    let minute_prefix = format!("rl:min:{}:", identifier);
    let minute_reqs = app_state.rate_limit_cache
        .iter()
        .filter(|(k, _)| k.starts_with(&minute_prefix))
        .count();
    
    const MAX_REQUESTS_PER_MINUTE: usize = 150;
    if minute_reqs > MAX_REQUESTS_PER_MINUTE {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    // 10-second burst limit (10 requests per 10 seconds)
    // Only check burst if we're under minute limit
    let burst_key = format!("rl:burst:{}:{}", identifier, request_id);
    app_state.rate_limit_cache.insert(burst_key, ()).await;
    
    let burst_prefix = format!("rl:burst:{}:", identifier);
    let burst_reqs = app_state.rate_limit_cache
        .iter()
        .filter(|(k, _)| k.starts_with(&burst_prefix))
        .count();
    
    const MAX_REQUESTS_PER_10SEC: usize = 10;
    if burst_reqs > MAX_REQUESTS_PER_10SEC {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}
