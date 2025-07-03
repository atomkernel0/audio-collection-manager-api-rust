use std::{env, net::SocketAddr};

use axum::{middleware, response::Response, Router};

use surrealdb::{
    engine::any::{self, Any},
    opt::auth::Root,
    Surreal,
};

pub use self::error::{Error, Result};

mod controllers;
mod error;
mod models;
mod routes;
mod services;
mod web;

#[derive(Clone)]
struct AppState {
    db: Surreal<Any>,
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

    let app_state = AppState { db };

    let routes_api = routes::albums::routes();

    let routes_all = Router::new()
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
