use std::{env, net::SocketAddr, time::Instant};

use axum::{
    body::Body,
    http::Request,
    middleware::{self, Next},
    response::Response,
    Router,
};
use chrono::Local;
use tower_http::cors::CorsLayer;

use surrealdb::{
    engine::any::{self, Any},
    opt::auth::Root,
    Surreal,
};

use crate::routes::playlist_routes::PlaylistRoutes;

pub use self::error::{Error, Result};

mod auth;
mod controllers;
mod error;
mod helpers;
mod models;
mod routes;
mod services;
mod web;

#[derive(Clone)]
struct AppState {
    db: Surreal<Any>,
    rate_limit_cache: moka::future::Cache<String, ()>,
}
#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let db_url = env::var("DB_URL")?;
    let db_ns = env::var("DB_NS")?;
    let db_name = env::var("DB_NAME")?;
    let db_user = env::var("DB_USER")?;
    let db_password = env::var("DB_PASSWORD")?;

    let db = any::connect(&db_url).await?;

    db.use_ns(&db_ns).use_db(&db_name).await?;

    db.signin(Root {
        username: &db_user,
        password: &db_password,
    })
    .await?;

    println!("Connected to DB!");

    let result = db
        .query("INFO FOR DB")
        .await
        .map_err(|e| Error::DbError(e.to_string()));

    println!("Database info: {:?}", result);

    let app_state = AppState {
        db,
        rate_limit_cache: moka::future::Cache::new(1000), // Stocke jusqu'à 1000 entrées
    };

    let song_routes = routes::song_routes::routes().route_layer(middleware::from_fn_with_state(
        app_state.clone(),
        web::mw_rate_limit::rate_limit_middleware,
    ));

    let routes_api = Router::new()
        .nest("/albums", routes::albums::routes())
        .merge(routes::artist::routes())
        .merge(song_routes)
        .merge(routes::auth_routes::routes())
        .merge(routes::user_routes::routes(app_state.clone()))
        .nest("/favorites", routes::favorites::routes())
        .nest("/playlist", PlaylistRoutes::routes())
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            web::mw_auth::mw_auth,
        ));

    let routes_all = Router::new()
        .merge(web::routes_login::routes())
        .nest("/api", routes_api)
        .with_state(app_state)
        .layer(middleware::from_fn(track_request_info))
        .layer(CorsLayer::very_permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("->> LISTENING on {addr}\n");

    axum::serve(
        listener,
        routes_all.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();

    Ok(())
}

async fn track_request_info(req: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();

    let res = next.run(req).await;

    let duration = start.elapsed();
    let timestamp = Local::now();
    let pid = std::process::id();
    let status = res.status();

    println!(
        "[{}] DEBUG ({}):\n    method: \"{}\"\n    path: \"{}\"\n    status: {}\n    duration: \"{:?}\"",
        timestamp.format("%H:%M:%S%.3f"),
        pid,
        method,
        path,
        status.as_u16(),
        duration
    );
    println!();

    res
}
