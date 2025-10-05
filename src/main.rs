use std::{env, net::SocketAddr, time::Duration};

use axum::{
    middleware::{self},
    Router,
    body::Body,
    http::{Request, Response},
};
use surrealdb::{
    engine::any::{self, Any},
    opt::auth::Root,
    Surreal,
};
use tower_http::{
    cors::CorsLayer, 
    trace::TraceLayer,
};
use tracing::Span;
use uuid::Uuid;

use crate::{
    auth::token_service::AuthConfig,
    routes::{
        album_routes::AlbumRoutes, artist_routes::ArtistRoutes, auth_routes::AuthRoutes,
        favorite_routes::FavoriteRoutes, playlist_routes::PlaylistRoutes,
        search_routes::SearchRoutes, song_routes::SongRoutes, user_routes::UserRoutes,
    },
};

pub use self::error::{Error, Result};

mod auth;
mod controllers;
mod error;
mod helpers;
mod models;
mod routes;
mod services;
mod middlewares;
mod validators;

#[derive(Clone)]
struct AppState {
    db: Surreal<Any>,
    #[allow(dead_code)]
    rate_limit_cache: moka::future::Cache<String, ()>,
    auth_config: AuthConfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    init_tracing();

    tracing::info!("Starting Audio Collection Manager API...");

    // Load environment variables
    let db_url = env::var("DB_URL")?;
    let db_ns = env::var("DB_NS")?;
    let db_name = env::var("DB_NAME")?;
    let db_user = env::var("DB_USER")?;
    let db_password = env::var("DB_PASSWORD")?;

    tracing::info!("Connecting to database at: {}", db_url);
    
    let db = any::connect(&db_url).await?;
    db.use_ns(&db_ns).use_db(&db_name).await?;
    db.signin(Root {
        username: &db_user,
        password: &db_password,
    })
    .await?;

    tracing::info!("Database connected successfully!");

    let auth_config = AuthConfig::from_env()?;
    tracing::info!("Auth configuration loaded");

    let app_state = AppState {
        db,
        rate_limit_cache: moka::future::Cache::new(1000),
        auth_config: auth_config.clone(),
    };

    let routes_api = Router::new()
        .nest("/auth", AuthRoutes::routes())
        .nest("/albums", AlbumRoutes::routes())
        .nest("/artists", ArtistRoutes::routes())
        .nest("/song", SongRoutes::routes())
        .nest("/search", SearchRoutes::routes())
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            middlewares::mw_rate_limit::rate_limit_middleware,
        ));

    let protected_routes = Router::new()
        .nest("/user", UserRoutes::routes())
        .nest("/favorites", FavoriteRoutes::routes())
        .nest("/playlist", PlaylistRoutes::routes())
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            middlewares::mw_auth::mw_auth,
        ))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            middlewares::mw_rate_limit::rate_limit_middleware,
        ));

    let routes_all = Router::new()
        .nest("/api", routes_api)
        .nest("/api", protected_routes)
        .with_state(app_state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    let request_id = Uuid::new_v4();
                    tracing::info_span!(
                        "http_request",
                        request_id = %request_id,
                        method = %request.method(),
                        path = %request.uri().path(),
                    )
                })
                .on_request(|request: &Request<Body>, _span: &Span| {
                    tracing::info!(
                        "{} {}",
                        request.method(),
                        request.uri().path()
                    );
                })
                .on_response(|response: &Response<Body>, latency: Duration, _span: &Span| {
                    let status = response.status();
                    let latency_ms = latency.as_millis();
                    
                    match status.as_u16() {
                        200..=299 => tracing::info!("{} ({}ms)", status, latency_ms),
                        400..=499 => tracing::warn!("⚠️ {} ({}ms)", status, latency_ms),
                        500..=599 => tracing::error!("❌ {} ({}ms)", status, latency_ms),
                        _ => tracing::info!("{} ({}ms)", status, latency_ms),
                    }
                })
        )
        .layer(CorsLayer::very_permissive());

    let host = env::var("BIND_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid bind address");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    tracing::info!("Listening on http://{}", addr);

    axum::serve(
        listener,
        routes_all.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            "audio_collection_manager_rust=debug,tower_http=info,info".into()
        });

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_file(true)
                .with_line_number(true)
                .compact()
        )
        .init();
}