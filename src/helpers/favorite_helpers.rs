use surrealdb::engine::any::Any;
use surrealdb::sql::Thing;
use surrealdb::{Error, Surreal};

use crate::models::{artist::Artist, song::Song};

/// Vérifie si un artiste existe dans la base de données
pub async fn artist_exists(db: &Surreal<Any>, artist_id: &str) -> Result<bool, Error> {
    let artist_thing = Thing::from(("artist", artist_id));
    let sql_query = "SELECT * FROM $artist_id;";
    let mut response = db
        .query(sql_query)
        .bind(("artist_id", artist_thing))
        .await?;
    let exists: Option<Artist> = response.take(0)?;
    Ok(exists.is_some())
}

/// Vérifie si un utilisateur existe dans la base de données
pub async fn user_exists(db: &Surreal<Any>, user_id: &str) -> Result<bool, Error> {
    let user_thing = Thing::from(("user", user_id));
    let sql_query = "SELECT * FROM $user_id;";
    let mut response = db.query(sql_query).bind(("user_id", user_thing)).await?;
    let exists: Option<serde_json::Value> = response.take(0)?;
    Ok(exists.is_some())
}

/// Vérifie si un album est déjà dans les favoris d'un utilisateur
pub async fn is_album_favorited(
    db: &Surreal<Any>,
    user_id: &str,
    album_id: &str,
) -> Result<bool, Error> {
    let user_thing = Thing::from(("user", user_id));
    let album_thing = Thing::from(("album", album_id));

    let sql_query = "SELECT count() as total FROM user_likes_album WHERE in = $user_id AND out = $album_id GROUP ALL";
    let mut response = db
        .query(sql_query)
        .bind(("user_id", user_thing))
        .bind(("album_id", album_thing))
        .await?;

    #[derive(serde::Deserialize)]
    struct CountResult {
        total: u64,
    }

    let result: Option<CountResult> = response.take(0)?;
    Ok(result.map(|r| r.total > 0).unwrap_or(false))
}

/// Vérifie si une chanson est déjà dans les favoris d'un utilisateur
pub async fn is_song_favorited(
    db: &Surreal<Any>,
    user_id: &str,
    song_id: &str,
) -> Result<bool, Error> {
    let user_thing = Thing::from(("user", user_id));
    let song_thing = Thing::from(("song", song_id));

    let sql_query = "SELECT count() as total FROM user_likes_song WHERE in = $user_id AND out = $song_id GROUP ALL";
    let mut response = db
        .query(sql_query)
        .bind(("user_id", user_thing))
        .bind(("song_id", song_thing))
        .await?;

    #[derive(serde::Deserialize)]
    struct CountResult {
        total: u64,
    }

    let result: Option<CountResult> = response.take(0)?;
    Ok(result.map(|r| r.total > 0).unwrap_or(false))
}

/// Vérifie si un artiste est déjà dans les favoris d'un utilisateur
pub async fn is_artist_favorited(
    db: &Surreal<Any>,
    user_id: &str,
    artist_id: &str,
) -> Result<bool, Error> {
    let user_thing = Thing::from(("user", user_id));
    let artist_thing = Thing::from(("artist", artist_id));

    let sql_query = "SELECT count() as total FROM user_likes_artist WHERE in = $user_id AND out = $artist_id GROUP ALL";
    let mut response = db
        .query(sql_query)
        .bind(("user_id", user_thing))
        .bind(("artist_id", artist_thing))
        .await?;

    #[derive(serde::Deserialize)]
    struct CountResult {
        total: u64,
    }

    let result: Option<CountResult> = response.take(0)?;
    Ok(result.map(|r| r.total > 0).unwrap_or(false))
}
