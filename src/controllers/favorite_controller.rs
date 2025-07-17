use crate::error::Error;
use surrealdb::{engine::any::Any, Surreal};

use crate::{models::favorite::*, services::favorite_service::FavoriteService};

pub async fn get_favorite_albums(
    db: &Surreal<Any>,
    user_id: &str,
    query: &FavoritesQuery,
) -> Result<FavoritesResponse<AlbumWithFavoriteMetadata>, Error> {
    FavoriteService::get_favorite_albums(db, user_id, query).await
}

pub async fn get_favorite_albums_count(db: &Surreal<Any>, user_id: &str) -> Result<u64, Error> {
    FavoriteService::get_favorite_albums_count(db, user_id).await
}

pub async fn get_is_favorite_album(
    db: &Surreal<Any>,
    album_id: &str,
    user_id: &str,
) -> Result<bool, Error> {
    FavoriteService::get_is_favorite_album(db, album_id, user_id).await
}

pub async fn get_favorite_songs(
    db: &Surreal<Any>,
    user_id: &str,
    query: &FavoritesQuery,
) -> Result<FavoritesResponse<SongWithFavoriteMetadata>, Error> {
    FavoriteService::get_favorite_songs(db, user_id, query).await
}

pub async fn get_favorite_artists(
    db: &Surreal<Any>,
    user_id: &str,
    query: &FavoritesQuery,
) -> Result<FavoritesResponse<ArtistWithFavoriteMetadata>, Error> {
    FavoriteService::get_favorite_artists(db, user_id, query).await
}

pub async fn add_favorite(
    db: &Surreal<Any>,
    user_id: &str,
    request: AddFavoriteRequest,
) -> Result<serde_json::Value, Error> {
    match request.item_type.as_str() {
        "album" => FavoriteService::add_favorite_album(db, user_id, request)
            .await
            .map(|res| serde_json::to_value(res).unwrap()),
        "song" => FavoriteService::add_favorite_song(db, user_id, request)
            .await
            .map(|res| serde_json::to_value(res).unwrap()),
        "artist" => FavoriteService::add_favorite_artist(db, user_id, request)
            .await
            .map(|res| serde_json::to_value(res).unwrap()),
        _ => Err(Error::DbError("Invalid Item".to_string())),
    }
}

pub async fn toggle_favorite_album(
    db: &Surreal<Any>,
    user_id: &str,
    album_id: &str,
) -> Result<bool, Error> {
    FavoriteService::toggle_favorite_album(db, user_id, album_id).await
}

pub async fn remove_favorite_album(
    db: &Surreal<Any>,
    user_id: &str,
    album_id: &str,
) -> Result<(), Error> {
    FavoriteService::remove_favorite_album(db, user_id, album_id).await
}

pub async fn update_favorite_album(
    db: &Surreal<Any>,
    user_id: &str,
    album_id: &str,
    request: UpdateFavoriteRequest,
) -> Result<AlbumWithFavoriteMetadata, Error> {
    FavoriteService::update_favorite_album(db, user_id, album_id, request).await
}

pub async fn get_favorites_statistics(
    db: &Surreal<Any>,
    user_id: &str,
) -> Result<FavoritesStatistics, Error> {
    FavoriteService::get_statistics(db, user_id).await
}
