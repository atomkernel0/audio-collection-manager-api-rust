use crate::{models::song::Song, Error};
use surrealdb::{engine::any::Any, sql::Thing, Surreal};

pub async fn song_exists(db: &Surreal<Any>, song_id: &str) -> Result<bool, Error> {
    let song_thing = Thing::from(("song", song_id));
    let sql_query = "SELECT * FROM $song_id;";
    let mut response = db.query(sql_query).bind(("song_id", song_thing)).await?;
    let exists: Option<Song> = response.take(0)?;
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

        let test_song_content = Song {
            id: None,
            title: "Titre".to_string(),
            file_url: "https://example.com/song/titre".to_string(),
            duration: Duration::from_secs(100),
            song_index: 1,
            tempo: 150.0,
            total_listens: 100,
            total_user_listens: 100,
            total_likes: 100,
        };

        let created_song: Song = db
            .create("song")
            .content(test_song_content)
            .await
            .unwrap()
            .expect("Test song creation returned nothing (None).");

        let song_id = created_song.id.unwrap().id.to_string();

        (db, song_id)
    }

    #[tokio::test]
    async fn test_song_exists() {
        let (db, valid_id) = setup_db().await;

        // --- Test 1: Verify that an existing song is detected ---
        let exists = song_exists(&db, &valid_id).await.unwrap();
        assert!(exists, "Song with ID '{}' should exist", valid_id);

        // --- Test 2: Verify that a valid but non-existent ID is detected ---
        let non_existent_id = "this_id_does_not_exist";
        let exists = song_exists(&db, non_existent_id).await.unwrap();
        assert!(!exists, "Song with a non-existent ID should not exist");

        // --- Test 3: Verify that a malformed ID is handled ---
        let malformed_id = "just_a_random_string";
        let exists = song_exists(&db, malformed_id).await.unwrap();
        assert!(!exists, "An song with a malformed ID should not exist");
    }
}
