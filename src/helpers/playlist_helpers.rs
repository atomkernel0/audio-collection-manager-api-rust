use surrealdb::engine::any::Any;
use surrealdb::sql::{Thing, Value};
use surrealdb::{Error, Surreal};

use crate::{
    helpers::thing_helpers::{create_playlist_thing, create_song_thing, create_user_thing},
    models::playlist::Playlist,
    Error as AppError,
};

/// Vérifie si une playlist existe et appartient à l'utilisateur
pub async fn playlist_exists_and_belongs_to_user(
    db: &Surreal<Any>,
    playlist_id: &str,
    user_id: &str,
) -> Result<bool, AppError> {
    let playlist_thing = create_playlist_thing(playlist_id);
    let user_thing = create_user_thing(user_id);

    let playlist_check: Option<Playlist> = db
        .query("SELECT * FROM playlist WHERE id = $playlist AND created_by = $user")
        .bind(("playlist", playlist_thing))
        .bind(("user", user_thing))
        .await?
        .take(0)?;

    Ok(playlist_check.is_some())
}

/// Vérifie si une chanson existe dans la base de données
pub async fn song_exists_in_db(db: &Surreal<Any>, song_id: &str) -> Result<bool, AppError> {
    let song_thing = create_song_thing(song_id);

    let song_check: Option<Value> = db
        .query("SELECT id FROM song WHERE id = $song")
        .bind(("song", song_thing))
        .await?
        .take(0)?;

    Ok(song_check.is_some())
}

/// Vérifie si une chanson est déjà dans une playlist
pub async fn song_already_in_playlist(
    db: &Surreal<Any>,
    playlist_id: &str,
    song_id: &str,
) -> Result<bool, AppError> {
    let playlist_thing = create_playlist_thing(playlist_id);
    let song_thing = create_song_thing(song_id);

    let existing: Option<Value> = db
        .query("SELECT * FROM playlist_contains_song WHERE in = $playlist AND out = $song")
        .bind(("playlist", playlist_thing))
        .bind(("song", song_thing))
        .await?
        .take(0)?;

    Ok(existing.is_some())
}

/// Vérifie si une playlist existe (sans vérifier l'appartenance)
pub async fn playlist_exists(db: &Surreal<Any>, playlist_id: &str) -> Result<bool, AppError> {
    let playlist_thing = create_playlist_thing(playlist_id);

    let playlist_check: Option<Value> = db
        .query("SELECT id FROM playlist WHERE id = $playlist")
        .bind(("playlist", playlist_thing))
        .await?
        .take(0)?;

    Ok(playlist_check.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use surrealdb::engine::any::connect;
    use surrealdb::sql::Duration;

    use crate::models::playlist::{CreatePlaylistRequest, Playlist};

    async fn setup_db() -> (Surreal<Any>, String, String) {
        let db = connect("mem://").await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();

        // Créer un utilisateur de test
        let user: Value = db
            .create("user")
            .content(serde_json::json!({
                "username": "testuser",
                "email": "test@example.com"
            }))
            .await
            .unwrap()
            .expect("Test user creation failed");

        let user_id = user.get("id").unwrap().to_string();

        // Créer une playlist de test
        let playlist_content = Playlist {
            id: None,
            name: "Test Playlist".to_string(),
            cover_url: Some("https://example.com/cover.jpg".to_string()),
            is_public: true,
            dominant_color: Some("#FF0000".to_string()),
            songs_count: 0,
            total_duration: Duration::from_secs(0),
            total_listens: 0,
            total_likes: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
            songs: None,
            created_by_username: None,
        };

        let created_playlist: Playlist = db
            .create("playlist")
            .content(playlist_content)
            .await
            .unwrap()
            .expect("Test playlist creation failed");

        let playlist_id = created_playlist.id.unwrap().id.to_string();

        // Créer une chanson de test
        let song_content = serde_json::json!({
            "title": "Test Song",
            "file_url": "https://example.com/song.mp3",
            "duration": Duration::from_secs(180),
            "song_index": 1,
            "tempo": 120.0,
            "total_listens": 100,
            "total_user_listens": 50,
            "total_likes": 25
        });

        let created_song: Value = db
            .create("song")
            .content(song_content)
            .await
            .unwrap()
            .expect("Test song creation failed");

        let song_id = created_song.get("id").unwrap().id.to_string();

        (db, playlist_id, song_id)
    }

    #[tokio::test]
    async fn test_playlist_exists_and_belongs_to_user() {
        let (db, playlist_id, user_id) = setup_db().await;

        // Test avec la bonne combinaison
        let exists = playlist_exists_and_belongs_to_user(&db, &playlist_id, &user_id)
            .await
            .unwrap();
        assert!(exists, "Playlist should exist and belong to user");

        // Test avec un mauvais utilisateur
        let exists = playlist_exists_and_belongs_to_user(&db, &playlist_id, "wrong_user")
            .await
            .unwrap();
        assert!(!exists, "Playlist should not belong to wrong user");
    }

    #[tokio::test]
    async fn test_song_exists_in_db() {
        let (db, _playlist_id, song_id) = setup_db().await;

        let exists = song_exists_in_db(&db, &song_id).await.unwrap();
        assert!(exists, "Song should exist");

        let exists = song_exists_in_db(&db, "non_existent_song").await.unwrap();
        assert!(!exists, "Non-existent song should not exist");
    }

    #[tokio::test]
    async fn test_song_already_in_playlist() {
        let (db, playlist_id, song_id) = setup_db().await;

        // D'abord, la chanson ne devrait pas être dans la playlist
        let exists = song_already_in_playlist(&db, &playlist_id, &song_id)
            .await
            .unwrap();
        assert!(!exists, "Song should not be in playlist initially");

        // Ajouter la chanson à la playlist
        let _ = db
            .query("RELATE $playlist->playlist_contains_song->$song")
            .bind(("playlist", create_playlist_thing(&playlist_id)))
            .bind(("song", create_song_thing(&song_id)))
            .await
            .unwrap();

        // Maintenant, la chanson devrait être dans la playlist
        let exists = song_already_in_playlist(&db, &playlist_id, &song_id)
            .await
            .unwrap();
        assert!(exists, "Song should be in playlist after adding");
    }
}
