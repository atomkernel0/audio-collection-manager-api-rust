use surrealdb::engine::any::Any;
use surrealdb::sql::Thing;
use surrealdb::{Error, Surreal};

use crate::models::album::Album;

/// Check if an album exists in the database
pub async fn album_exists(db: &Surreal<Any>, album_id: &str) -> Result<bool, Error> {
    let album_thing = Thing::from(("album", album_id));
    let sql_query = "SELECT * FROM $album_id;";
    let mut response = db.query(sql_query).bind(("album_id", album_thing)).await?;
    let exists: Option<Album> = response.take(0)?;
    Ok(exists.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealdb::engine::any::connect;
    use surrealdb::sql::Duration;

    async fn setup_db() -> (Surreal<Any>, String) {
        let db = connect("mem://").await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();

        let test_album_content = Album {
            id: None,
            title: "AlbumTitle".to_string(),
            cover_url: Some("https://example.com/cover.jpg".to_string()),
            release_year: Some(2001),
            genres: vec!["RAP".to_string(), "ROCK_PSY".to_string()],
            langs: vec!["en".to_string()],
            dominant_color: Some("#D4AF37".to_string()),
            total_tracks: 14,
            total_duration: Duration::from_secs(100),
            total_listens: 999999,
            total_user_listens: 0,
            total_likes: 500000,
        };

        let created_album: Album = db
            .create("album")
            .content(test_album_content)
            .await
            .unwrap()
            .expect("Test album creation returned nothing (None).");

        let album_id = created_album.id.unwrap().id.to_string();

        (db, album_id)
    }

    #[tokio::test]
    async fn test_album_exists() {
        let (db, valid_id) = setup_db().await;

        // --- Test 1: Verify that an existing album is detected ---
        let exists = album_exists(&db, &valid_id).await.unwrap();
        assert!(exists, "Album with ID '{}' should exist", valid_id);

        // --- Test 2: Verify that a valid but non-existent ID is detected ---
        let non_existent_id = "this_id_does_not_exist";
        let exists = album_exists(&db, non_existent_id).await.unwrap();
        assert!(!exists, "Album with a non-existent ID should not exist");

        // --- Test 3: Verify that a malformed ID is handled ---
        let malformed_id = "just_a_random_string";
        let exists = album_exists(&db, malformed_id).await.unwrap();
        assert!(!exists, "An album with a malformed ID should not exist");
    }
}
