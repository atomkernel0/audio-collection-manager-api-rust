use std::net::SocketAddr;

use axum::{
    extract::{Path, Query},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Router,
};

use model::album::AlbumModelController;

use serde::Deserialize;
use tower_http::services::ServeDir;

pub use self::error::{Error, Result};

mod error;
mod model;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
    let album_mc = AlbumModelController::new().await?;

    let routes_api = web::routes_albums::routes(album_mc.clone())
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .nest("/api", routes_api)
        .layer(middleware::map_response(main_response_mapper))
        .fallback_service(routes_static());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("->> LISTENING on {addr}\n");

    axum::serve(listener, routes_all.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn main_response_mapper(res: Response) -> Response {
    println!("->> {:<12} - main_reponse_mapper", "RES_MAPPER");

    println!();
    res
}

fn routes_static() -> Router {
    Router::new().fallback_service(get_service(ServeDir::new("./")))
}

fn routes_hello() -> Router {
    Router::new()
        .route("/hello", get(handler_hello))
        .route("/hello2/{name}", get(handler_hello2))
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

// `/hello?name=allo`
async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    println!("->> {:<12} - handler-hello - {params:?}", "HANDLER");

    let name = params.name.as_deref().unwrap_or("World!");
    Html(format!("Bo <strong>{name}</string>"))
}

// `/hello2/allo`
async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
    println!("->> {:<12} - handler-hello - {name:?}", "HANDLER");

    Html(format!("Bo <strong>{name}</string>"))
}
