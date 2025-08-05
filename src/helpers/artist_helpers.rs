use surrealdb::engine::any::Any;
use surrealdb::{Error, Surreal};

use crate::helpers::thing_helpers::create_artist_thing;
use crate::models::artist::Artist;

/// Check if an artist exists in the database
pub async fn artist_exists(db: &Surreal<Any>, artist_id: &str) -> Result<bool, Error> {
    let artist_thing = create_artist_thing(artist_id);
    let sql_query = "SELECT * FROM $artist_id;";
    let mut response = db
        .query(sql_query)
        .bind(("artist_id", artist_thing))
        .await?;
    let exists: Option<Artist> = response.take(0)?;
    Ok(exists.is_some())
}

#[cfg(test)]
mod tests {
    use crate::models::music_genre::MusicGenre;

    use super::*;
    use surrealdb::engine::any::connect;

    async fn setup_db() -> (Surreal<Any>, String) {
        let db = connect("mem://").await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();

        let test_artist_content = Artist {
            id: None,
            name: "Artiste".to_string(),
            genres: vec![MusicGenre::Rap, MusicGenre::PsychedelicRock],
            country_code: "fr".to_string(),
            artist_image: Some("https://example.com/artist.jpg".to_string()),
            albums_count: 3,
            songs_count: 10,
            total_likes: 0,
        };

        let created_artist: Artist = db
            .create("artist")
            .content(test_artist_content)
            .await
            .unwrap()
            .expect("Test artist creation returned nothing (None).");

        let artist_id = created_artist.id.unwrap().id.to_string();

        (db, artist_id)
    }

    #[tokio::test]
    async fn test_artist_exists() {
        let (db, valid_id) = setup_db().await;

        // --- Test 1: Verify that an existing artist is detected ---
        let exists = artist_exists(&db, &valid_id).await.unwrap();
        assert!(exists, "Artist with ID '{}' should exist", valid_id);

        // --- Test 2: Verify that a valid but non-existent ID is detected ---
        let non_existent_id = "this_id_does_not_exist";
        let exists = artist_exists(&db, non_existent_id).await.unwrap();
        assert!(!exists, "Artist with a non-existent ID should not exist");

        // --- Test 3: Verify that a malformed ID is handled ---
        let malformed_id = "just_a_random_string";
        let exists = artist_exists(&db, malformed_id).await.unwrap();
        assert!(!exists, "An artist with a malformed ID should not exist");
    }
}
