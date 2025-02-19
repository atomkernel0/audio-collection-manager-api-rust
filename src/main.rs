use std::net::SocketAddr;

use axum::{
    extract::{Path, Query},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};

use serde::Deserialize;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};

pub use self::error::{Error, Result};

mod controllers;
mod error;
mod models;
mod routes;
mod web;

#[derive(Clone)]
struct AppState {
    db: Surreal<Client>,
}
#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;

    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    db.use_ns("music").use_db("album").await?;
    println!("Connected to DB!");

    let result = db
        .query("INFO FOR DB")
        .await
        .map_err(|e| Error::DbError(e.to_string()));

    println!("Database info: {:?}", result);

    let app_state = AppState { db };

    let routes_api = routes::albums::routes();

    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .nest("/api", routes_api)
        .with_state(app_state)
        .layer(middleware::map_response(main_response_mapper));

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

fn routes_hello() -> Router<AppState> {
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
