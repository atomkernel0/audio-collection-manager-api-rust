use crate::{Error, Result};
use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use lazy_regex::regex_captures;

pub async fn mw_require_auth(req: Request<Body>, next: Next) -> Result<Response> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .filter(|&h| h == "lol");

    auth_header.ok_or(Error::AuthFailNoAuthTokenCookie)?;

    Ok(next.run(req).await)
}

fn parse_token(token: String) -> Result<(u64, String, String)> {
    let (_whole, user_id, exp, sign) = regex_captures!(r#"^user-(\d+)\.(.+)\.(.+)"#, &token)
        .ok_or(Error::AuthFailTokenWrongFormat)?;

    let user_id: u64 = user_id
        .parse()
        .map_err(|_| Error::AuthFailTokenWrongFormat)?;

    Ok((user_id, exp.to_string(), sign.to_string()))
}
