use crate::{
    controllers::{self, favorite_controller},
    error::Result,
    models::favorite::*,
    web::mw_auth::Ctx,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Extension, Json, Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/albums", get(get_favorite_albums_handler))
        .route(
            "/albums/{album_id}/check",
            get(get_is_favorite_album_handler),
        )
        .route("/albums/count", get(get_favorite_albums_count))
        .route(
            "/albums/{album_id}/toggle",
            post(toggle_favorite_album_handler),
        )
        .route("/songs", get(get_favorite_songs_handler))
        .route("/artists", get(get_favorite_artists_handler))
        .route("/add", post(add_favorite_handler))
        .route(
            "/albums/{album_id}/remove",
            delete(remove_favorite_album_handler),
        )
        .route(
            "/albums/{album_id}/update",
            patch(update_favorite_album_handler),
        )
        .route("/statistics", get(get_favorites_statistics_handler))
}

async fn get_favorite_albums_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
    Query(query): Query<FavoritesQuery>,
) -> Result<Json<FavoritesResponse<AlbumWithFavoriteMetadata>>> {
    let albums =
        controllers::favorite_controller::get_favorite_albums(&state.db, &ctx.user_id, &query)
            .await?;
    Ok(Json(albums))
}

async fn get_favorite_albums_count(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
) -> Result<Json<u64>> {
    let count =
        controllers::favorite_controller::get_favorite_albums_count(&state.db, &ctx.user_id)
            .await?;
    Ok(Json(count))
}

async fn get_is_favorite_album_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
    Path(album_id): Path<String>,
) -> Result<Json<bool>> {
    let is_favorite =
        controllers::favorite_controller::get_is_favorite_album(&state.db, &album_id, &ctx.user_id)
            .await?;

    Ok(Json(is_favorite))
}

async fn get_favorite_songs_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
    Query(query): Query<FavoritesQuery>,
) -> Result<Json<FavoritesResponse<SongWithFavoriteMetadata>>> {
    let songs =
        controllers::favorite_controller::get_favorite_songs(&state.db, &ctx.user_id, &query)
            .await?;
    Ok(Json(songs))
}

async fn get_favorite_artists_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
    Query(query): Query<FavoritesQuery>,
) -> Result<Json<FavoritesResponse<ArtistWithFavoriteMetadata>>> {
    let artists =
        controllers::favorite_controller::get_favorite_artists(&state.db, &ctx.user_id, &query)
            .await?;
    Ok(Json(artists))
}

pub async fn toggle_favorite_album_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
    Path(album_id): Path<String>,
) -> Result<(StatusCode, Json<bool>)> {
    let liked =
        favorite_controller::toggle_favorite_album(&state.db, &ctx.user_id, &album_id).await?;
    let status_code = if liked {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status_code, Json(liked)))
}

async fn add_favorite_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
    Json(request): Json<AddFavoriteRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let result =
        controllers::favorite_controller::add_favorite(&state.db, &ctx.user_id, request).await?;
    Ok((StatusCode::CREATED, Json(result)))
}

async fn remove_favorite_album_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
    Path(album_id): Path<String>,
) -> Result<StatusCode> {
    controllers::favorite_controller::remove_favorite_album(&state.db, &ctx.user_id, &album_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_favorite_album_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
    Path(album_id): Path<String>,
    Json(request): Json<UpdateFavoriteRequest>,
) -> Result<Json<AlbumWithFavoriteMetadata>> {
    let album = controllers::favorite_controller::update_favorite_album(
        &state.db,
        &ctx.user_id,
        &album_id,
        request,
    )
    .await?;
    Ok(Json(album))
}

async fn get_favorites_statistics_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<Ctx>,
) -> Result<Json<FavoritesStatistics>> {
    let stats =
        controllers::favorite_controller::get_favorites_statistics(&state.db, &ctx.user_id).await?;
    Ok(Json(stats))
}
