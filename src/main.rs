use std::{env, net::SocketAddr};

use axum::{
    middleware::{self},
    Router,
};
use surrealdb::{
    engine::any::{self, Any},
    opt::auth::Root,
    Surreal,
};

use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    auth::token_service::AuthConfig,
    routes::{
        album_routes::AlbumRoutes, artist_routes::ArtistRoutes, auth_routes::AuthRoutes,
        favorites::FavoriteRoutes, playlist_routes::PlaylistRoutes, search_routes::SearchRoutes,
        song_routes::SongRoutes, user_routes::UserRoutes,
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
mod web;

#[derive(Clone)]
struct AppState {
    db: Surreal<Any>,
    #[allow(dead_code)]
    rate_limit_cache: moka::future::Cache<String, ()>, //TODO: implement feature
    auth_config: AuthConfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

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

    let auth_config = AuthConfig::from_env().expect("Failed to load auth configuration");

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
        .nest("/user", UserRoutes::routes())
        .nest("/favorites", FavoriteRoutes::routes())
        .nest("/search", SearchRoutes::routes())
        .nest("/playlist", PlaylistRoutes::routes())
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            web::mw_auth::mw_auth,
        ));

    let routes_all = Router::new()
        .nest("/api", routes_api)
        .with_state(app_state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::very_permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("--> LISTENING on {addr}\n");

    axum::serve(
        listener,
        routes_all.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();

    Ok(())
}
