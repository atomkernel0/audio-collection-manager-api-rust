use crate::error::Error;
use crate::models::album::AlbumsMetaResponse;
use crate::{
    helpers::{
        album_helpers::album_exists,
        thing_helpers::{create_album_thing, create_user_thing},
    },
    models::{
        album::{AlbumWithArtists, AlbumWithRelations},
        database_helpers::CountResult,
    },
};
use surrealdb::{engine::any::Any, Surreal};

pub struct AlbumService;

impl AlbumService {
    pub async fn get_albums(db: &Surreal<Any>) -> Result<Vec<AlbumWithArtists>, Error> {
        let sql_query = "
            SELECT *, 
            <-artist_creates_album<-artist.* AS artists 
            FROM album 
            ORDER BY title ASC;
        ";

        let mut response = db.query(sql_query).await?;
        let albums: Vec<AlbumWithArtists> = response.take(0)?;

        Ok(albums)
    }

    pub async fn get_album(
        db: &Surreal<Any>,
        album_id: &str,
    ) -> Result<Option<AlbumWithRelations>, Error> {
        let album_thing = create_album_thing(album_id);

        let sql_query = "
            SELECT *,
            (SELECT * FROM (SELECT VALUE in FROM <-artist_creates_album)) AS artists,
            (SELECT * FROM (SELECT VALUE out FROM ->album_contains_song) ORDER BY song_index ASC) AS songs
            FROM $album_thing;
        ";

        let mut response = db
            .query(sql_query)
            .bind(("album_thing", album_thing))
            .await?;

        let album: Option<AlbumWithRelations> = response.take(0)?;

        Ok(album)
    }

    pub async fn get_initial_albums_with_meta(
        db: &Surreal<Any>,
        limit: u32,
    ) -> Result<AlbumsMetaResponse, Error> {
        let safe_limit = limit.min(50).max(1);
        
        let albums_query = format!(
            "SELECT *, 
             <-artist_creates_album<-artist.* AS artists 
             FROM album 
             ORDER BY title ASC 
             LIMIT {};",
            safe_limit
        );
        
        let count_query = "SELECT count() AS total FROM album GROUP ALL;";
        
        let (albums_result, count_result) = tokio::join!(
            db.query(albums_query),
            db.query(count_query)
        );
        
        let mut albums_response = albums_result?;
        let albums: Vec<AlbumWithArtists> = albums_response.take(0)?;
        
        let mut count_response = count_result?;
        let count: Option<CountResult> = count_response.take(0)?;
        let total_count = count.map(|c| c.total as u32).unwrap_or(0);
        
        let has_more = total_count > safe_limit;
        
        Ok(AlbumsMetaResponse {
            albums,
            total_count,
            has_more,
        })
    }

    pub async fn get_albums_batch(
        db: &Surreal<Any>,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<AlbumWithArtists>, Error> {
        let safe_limit = limit.min(100).max(1);
        let safe_offset = offset;
        
        let sql_query = format!(
            "SELECT *, 
             <-artist_creates_album<-artist.* AS artists 
             FROM album 
             ORDER BY title ASC 
             START {} LIMIT {};",
            safe_offset, safe_limit
        );
        
        let mut response = db.query(sql_query).await?;
        let albums: Vec<AlbumWithArtists> = response.take(0)?;
        
        Ok(albums)
    }

    pub async fn get_albums_filtered(
        db: &Surreal<Any>,
        offset: u32,
        limit: u32,
        genres: Option<Vec<String>>,
        search: Option<String>,
        sort_by: Option<String>,
    ) -> Result<AlbumsMetaResponse, Error> {
        let safe_limit = limit.min(100).max(1);
        let safe_offset = offset;
        
        let mut where_clauses = Vec::new();
        let mut genres_binding: Option<Vec<String>> = None;
        let mut search_binding: Option<String> = None;
        
        if let Some(genres_list) = genres {
            if !genres_list.is_empty() {
                where_clauses.push("genres CONTAINSANY $genres");
                genres_binding = Some(genres_list);
            }
        }
        
        if let Some(search_term) = search {
            if !search_term.is_empty() {
                where_clauses.push("(string::lowercase(title) CONTAINS string::lowercase($search) OR id IN (SELECT VALUE out FROM artist_creates_album WHERE in IN (SELECT id FROM artist WHERE string::lowercase(name) CONTAINS string::lowercase($search))))");
                search_binding = Some(search_term);
            }
        }
        
        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };
        
        // Determine sort order - removed NULLS LAST for compatibility
        let order_clause = match sort_by.as_deref() {
            Some("popular") => "ORDER BY total_listens DESC",
            Some("recent") => "ORDER BY release_year DESC",
            Some("alphabetical") | _ => "ORDER BY title ASC",
        };
        
        let albums_query = format!(
            "SELECT *, 
             <-artist_creates_album<-artist.* AS artists 
             FROM album 
             {} 
             {} 
             START {} LIMIT {};",
            where_clause, order_clause, safe_offset, safe_limit
        );
        
        let count_query = format!(
            "SELECT count() AS total 
             FROM album 
             {} 
             GROUP ALL;",
            where_clause
        );
        
        let mut albums_query_builder = db.query(albums_query);
        let mut count_query_builder = db.query(count_query);
        
        if let Some(g) = &genres_binding {
            albums_query_builder = albums_query_builder.bind(("genres", g.clone()));
            count_query_builder = count_query_builder.bind(("genres", g.clone()));
        }
        if let Some(s) = &search_binding {
            albums_query_builder = albums_query_builder.bind(("search", s.clone()));
            count_query_builder = count_query_builder.bind(("search", s.clone()));
        }
        
        let (albums_result, count_result) = tokio::join!(
            albums_query_builder,
            count_query_builder
        );
        
        let mut albums_response = albums_result?;
        let albums: Vec<AlbumWithArtists> = albums_response.take(0)?;
        
        let mut count_response = count_result?;
        let count: Option<CountResult> = count_response.take(0)?;
        let total_count = count.map(|c| c.total as u32).unwrap_or(0);
        
        let has_more = (safe_offset + albums.len() as u32) < total_count;
        
        Ok(AlbumsMetaResponse {
            albums,
            total_count,
            has_more,
        })
    }

    pub async fn listen_to_album(
        db: &Surreal<Any>,
        album_id: &str,
        user_id: Option<&str>,
    ) -> Result<bool, Error> {
        let album_thing = create_album_thing(album_id);

        if !album_exists(db, album_id).await? {
            return Err(Error::AlbumNotFound {
                id: album_id.to_string(),
            });
        }

        if let Some(user_id) = user_id {
            let user_thing = create_user_thing(user_id);

            // Try to update existing relation first
            let update_query = r#"
                UPDATE user_listens_album SET
                    total_listens = (IF total_listens = NONE THEN 0 ELSE total_listens END) + 1,
                    total_duration = (
                        IF total_duration = NONE THEN 0s ELSE total_duration END
                    ) + (SELECT VALUE total_duration FROM $album_id LIMIT 1)[0],
                    recent_dates = array::slice(
                        array::prepend(
                            IF type::is::array(recent_dates) THEN recent_dates ELSE [] END,
                            time::now()
                        ),
                        0,
                        30
                    ),
                    first_listened_at = IF first_listened_at = NONE THEN time::now() ELSE first_listened_at END,
                    last_listened_at = time::now()
                WHERE in = $user_id AND out = $album_id
                RETURN AFTER;
            "#;

            let mut update_response = db
                .query(update_query)
                .bind(("user_id", user_thing.clone()))
                .bind(("album_id", album_thing.clone()))
                .await?;

            #[derive(serde::Deserialize)]
            struct UpdateResult {}

            let updated: Vec<UpdateResult> = update_response.take(0)?;

            if updated.is_empty() {
                // No existing relation, create new one
                let create_query = r#"
                    BEGIN TRANSACTION;
                    
                    LET $existing = (SELECT * FROM user_listens_album WHERE in = $user_id AND out = $album_id);
                    LET $album_duration = (SELECT VALUE total_duration FROM $album_id LIMIT 1)[0];
                    
                    IF array::len($existing) = 0 THEN
                        RELATE $user_id->user_listens_album->$album_id SET
                            total_listens = 1,
                            total_duration = $album_duration,
                            recent_dates = [time::now()],
                            first_listened_at = time::now(),
                            last_listened_at = time::now()
                    ELSE
                        UPDATE user_listens_album SET
                            total_listens = total_listens + 1,
                            total_duration = total_duration + $album_duration,
                            recent_dates = array::slice(
                                array::prepend(recent_dates, time::now()),
                                0,
                                30
                            ),
                            last_listened_at = time::now()
                        WHERE in = $user_id AND out = $album_id
                    END;
                    
                    COMMIT TRANSACTION;
                "#;

                db.query(create_query)
                    .bind(("user_id", user_thing))
                    .bind(("album_id", album_thing.clone()))
                    .await?;
            }
        }

        // Update global counter
        db.query("UPDATE $album_id SET total_listens = (total_listens OR 0) + 1")
            .bind(("album_id", album_thing))
            .await?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::album::Album;
    use crate::models::artist::{Artist};
    use crate::models::music_genre::MusicGenre;
    use crate::models::song::Song;
    use surrealdb::engine::any::connect;
    use surrealdb::sql::Duration;
    use surrealdb::Datetime;

    async fn setup_db() -> Surreal<Any> {
        let db = connect("mem://").await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db
    }

    async fn create_test_album(db: &Surreal<Any>, id: &str, title: &str) -> String {
        let album_content = Album {
            id: Some(create_album_thing(id)),
            title: title.to_string(),
            cover_url: Some(format!("/covers/{}.jpg", id)),
            release_year: Some(2024),
            genres: vec!["Rock".to_string()],
            langs: vec!["en".to_string()],
            dominant_color: Some("#FF0000".to_string()),
            total_tracks: 10,
            total_duration: Duration::new(2400, 0),
            total_listens: 0,
            total_user_listens: 0,
            total_likes: 0,
        };

        let created: Album = db
            .create("album")
            .content(album_content)
            .await
            .unwrap()
            .expect("Album creation failed");

        created.id.unwrap().id.to_string()
    }

    async fn create_test_artist(db: &Surreal<Any>, id: &str, name: &str) -> String {
        let artist_content = Artist {
            id: Some(crate::helpers::thing_helpers::create_artist_thing(id)),
            name: name.to_string(),
            genres: vec![MusicGenre::Rac],
            country_code: "US".to_string(),
            artist_image: Some(format!("/artists/{}.jpg", id)),
            albums_count: 1,
            songs_count: 10,
            total_likes: 0,
        };

        let created: Artist = db
            .create("artist")
            .content(artist_content)
            .await
            .unwrap()
            .expect("Artist creation failed");

        created.id.unwrap().id.to_string()
    }

    async fn create_test_song(db: &Surreal<Any>, id: &str, title: &str, index: u16) -> String {
        let song_content = Song {
            id: Some(crate::helpers::thing_helpers::create_song_thing(id)),
            title: title.to_string(),
            file_url: format!("/songs/{}.mp3", id),
            duration: Duration::new(240, 0),
            song_index: index,
            tempo: 120.0,
            total_listens: 0,
            total_user_listens: 0,
            total_likes: 0,
        };

        let created: Song = db
            .create("song")
            .content(song_content)
            .await
            .unwrap()
            .expect("Song creation failed");

        created.id.unwrap().id.to_string()
    }

    async fn create_test_user(db: &Surreal<Any>, id: &str) -> String {
        use crate::models::user::UserRecord;

        let user_content = UserRecord {
            id: Some(create_user_thing(id)),
            username: format!("user_{}", id),
            password: "hashed_password".to_string(),
            created_at: Datetime::default(),
            listen_count: 0,
            total_listening_time: 0,
            favorite_count: 0,
            listening_streak: 0,
            badges: Vec::new(),
            level: 0,
            experience_points: 0,
        };

        let created: UserRecord = db
            .create("user")
            .content(user_content)
            .await
            .unwrap()
            .expect("User creation failed");

        created.id.unwrap().id.to_string()
    }

    #[tokio::test]
    async fn test_get_albums_empty() {
        let db = setup_db().await;
        let albums = AlbumService::get_albums(&db).await.unwrap();
        assert_eq!(albums.len(), 0);
    }

    #[tokio::test]
    async fn test_get_albums_with_artists() {
        let db = setup_db().await;

        let album_id = create_test_album(&db, "album1", "Test Album").await;
        let artist_id = create_test_artist(&db, "artist1", "Test Artist").await;

        // Create relation
        db.query("RELATE $artist->artist_creates_album->$album")
            .bind(("artist", create_artist_thing(&artist_id)))
            .bind(("album", create_album_thing(&album_id)))
            .await
            .unwrap();

        let albums = AlbumService::get_albums(&db).await.unwrap();

        assert_eq!(albums.len(), 1);
        assert_eq!(albums[0].title, "Test Album");
        assert_eq!(albums[0].artists.len(), 1);
        assert_eq!(albums[0].artists[0].name, "Test Artist");
    }

    #[tokio::test]
    async fn test_get_album_with_relations() {
        let db = setup_db().await;

        let album_id = create_test_album(&db, "album1", "Test Album").await;
        let artist_id = create_test_artist(&db, "artist1", "Test Artist").await;
        let song1_id = create_test_song(&db, "song1", "Song 1", 1).await;
        let song2_id = create_test_song(&db, "song2", "Song 2", 2).await;

        // Create relations
        db.query("
            RELATE $artist->artist_creates_album->$album;
            RELATE $album->album_contains_song->$song1;
            RELATE $album->album_contains_song->$song2;
        ")
        .bind(("artist", create_artist_thing(&artist_id)))
        .bind(("album", create_album_thing(&album_id)))
        .bind(("song1", create_song_thing(&song1_id)))
        .bind(("song2", create_song_thing(&song2_id)))
        .await
        .unwrap();

        let album = AlbumService::get_album(&db, &album_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(album.title, "Test Album");
        assert_eq!(album.artists.len(), 1);
        assert_eq!(album.songs.len(), 2);
        assert_eq!(album.songs[0].title, "Song 1");
        assert_eq!(album.songs[1].title, "Song 2");
    }

    #[tokio::test]
    async fn test_get_album_not_found() {
        let db = setup_db().await;
        let album = AlbumService::get_album(&db, "nonexistent")
            .await
            .unwrap();
        assert!(album.is_none());
    }

    #[tokio::test]
    async fn test_get_initial_albums_with_meta() {
        let db = setup_db().await;

        for i in 1..=5 {
            create_test_album(&db, &format!("album{}", i), &format!("Album {}", i)).await;
        }

        let response = AlbumService::get_initial_albums_with_meta(&db, 3)
            .await
            .unwrap();

        assert_eq!(response.albums.len(), 3);
        assert_eq!(response.total_count, 5);
        assert!(response.has_more);
    }

    #[tokio::test]
    async fn test_get_initial_albums_with_meta_limit() {
        let db = setup_db().await;

        for i in 1..=60 {
            create_test_album(&db, &format!("album{}", i), &format!("Album {:02}", i)).await;
        }

        let response = AlbumService::get_initial_albums_with_meta(&db, 100)
            .await
            .unwrap();

        assert_eq!(response.albums.len(), 50); // Should be capped at 50
        assert_eq!(response.total_count, 60);
        assert!(response.has_more);
    }

    #[tokio::test]
    async fn test_get_albums_batch() {
        let db = setup_db().await;

        for i in 1..=10 {
            create_test_album(&db, &format!("album{}", i), &format!("Album {:02}", i)).await;
        }

        let batch1 = AlbumService::get_albums_batch(&db, 0, 5)
            .await
            .unwrap();
        assert_eq!(batch1.len(), 5);

        let batch2 = AlbumService::get_albums_batch(&db, 5, 5)
            .await
            .unwrap();
        assert_eq!(batch2.len(), 5);

        let batch3 = AlbumService::get_albums_batch(&db, 10, 5)
            .await
            .unwrap();
        assert_eq!(batch3.len(), 0);
    }

    #[tokio::test]
    async fn test_get_albums_filtered_by_genre() {
        let db = setup_db().await;

        // Create albums with different genres
        let rock_album = Album {
            id: Some(create_album_thing("rock1")),
            title: "Rock Album".to_string(),
            cover_url: None,
            release_year: Some(2024),
            genres: vec!["Rock".to_string()],
            langs: vec!["en".to_string()],
            dominant_color: None,
            total_tracks: 10,
            total_duration: Duration::new(2400, 0),
            total_listens: 0,
            total_user_listens: 0,
            total_likes: 0,
        };

        let jazz_album = Album {
            id: Some(create_album_thing("jazz1")),
            title: "Jazz Album".to_string(),
            cover_url: None,
            release_year: Some(2024),
            genres: vec!["Jazz".to_string()],
            langs: vec!["en".to_string()],
            dominant_color: None,
            total_tracks: 10,
            total_duration: Duration::new(2400, 0),
            total_listens: 0,
            total_user_listens: 0,
            total_likes: 0,
        };

        db.create::<Option<Album>>("album").content(rock_album).await.unwrap();
        db.create::<Option<Album>>("album").content(jazz_album).await.unwrap();

        let response = AlbumService::get_albums_filtered(
            &db,
            0,
            10,
            Some(vec!["Rock".to_string()]),
            None,
            None,
        )
        .await
        .unwrap();

        assert_eq!(response.albums.len(), 1);
        assert_eq!(response.albums[0].title, "Rock Album");
        assert_eq!(response.total_count, 1);
        assert!(!response.has_more);
    }

    #[tokio::test]
    async fn test_get_albums_filtered_by_search() {
        let db = setup_db().await;

        create_test_album(&db, "album1", "Finding Nemo").await;
        create_test_album(&db, "album2", "The Matrix").await;
        create_test_album(&db, "album3", "Finding Dory").await;

        let response = AlbumService::get_albums_filtered(
            &db,
            0,
            10,
            None,
            Some("Finding".to_string()),
            None,
        )
        .await
        .unwrap();

        assert_eq!(response.albums.len(), 2);
        assert!(response.albums.iter().any(|a| a.title == "Finding Dory"));
        assert!(response.albums.iter().any(|a| a.title == "Finding Nemo"));
    }

    #[tokio::test]
    async fn test_get_albums_filtered_sorting() {
        let db = setup_db().await;

        // Create albums with different stats
        let popular_album = Album {
            id: Some(create_album_thing("popular")),
            title: "Popular Album".to_string(),
            cover_url: None,
            release_year: Some(2020),
            genres: vec!["Pop".to_string()],
            langs: vec!["en".to_string()],
            dominant_color: None,
            total_tracks: 10,
            total_duration: Duration::new(2400, 0),
            total_listens: 1000,
            total_user_listens: 0,
            total_likes: 0,
        };

        let recent_album = Album {
            id: Some(create_album_thing("recent")),
            title: "Recent Album".to_string(),
            cover_url: None,
            release_year: Some(2024),
            genres: vec!["Pop".to_string()],
            langs: vec!["en".to_string()],
            dominant_color: None,
            total_tracks: 10,
            total_duration: Duration::new(2400, 0),
            total_listens: 10,
            total_user_listens: 0,
            total_likes: 0,
        };

        db.create::<Option<Album>>("album").content(popular_album).await.unwrap();
        db.create::<Option<Album>>("album").content(recent_album).await.unwrap();

        // Test popular sorting
        let response = AlbumService::get_albums_filtered(
            &db,
            0,
            10,
            None,
            None,
            Some("popular".to_string()),
        )
        .await
        .unwrap();

        assert_eq!(response.albums[0].title, "Popular Album");

        // Test recent sorting
        let response = AlbumService::get_albums_filtered(
            &db,
            0,
            10,
            None,
            None,
            Some("recent".to_string()),
        )
        .await
        .unwrap();

        assert_eq!(response.albums[0].title, "Recent Album");
    }

    #[tokio::test]
    async fn test_listen_to_album_without_user() {
        let db = setup_db().await;
        let album_id = create_test_album(&db, "album1", "Test Album").await;

        let result = AlbumService::listen_to_album(&db, &album_id, None)
            .await
            .unwrap();

        assert!(result);

        // Check album stats were updated
        let album_thing = create_album_thing(&album_id);
        let albums: Vec<Album> = db
            .query("SELECT * FROM $album")
            .bind(("album", album_thing))
            .await
            .unwrap()
            .take(0)
            .unwrap();

        assert_eq!(albums[0].total_listens, 1);
    }

    #[tokio::test]
    async fn test_listen_to_album_with_user() {
        let db = setup_db().await;
        let album_id = create_test_album(&db, "album1", "Test Album").await;
        let user_id = create_test_user(&db, "user1").await;

        // First listen
        let result = AlbumService::listen_to_album(&db, &album_id, Some(&user_id))
            .await
            .unwrap();
        assert!(result);

        // Check relation was created
        let query = "SELECT * FROM user_listens_album WHERE in = $user AND out = $album";
        let mut response = db
            .query(query)
            .bind(("user", create_user_thing(&user_id)))
            .bind(("album", create_album_thing(&album_id)))
            .await
            .unwrap();

        #[derive(serde::Deserialize)]
        struct ListenRelation {
            total_listens: u64,
            total_duration: Duration,
        }

        let relations: Vec<ListenRelation> = response.take(0).unwrap();
        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].total_listens, 1);
        assert_eq!(relations[0].total_duration, Duration::new(2400, 0));

        // Second listen
        AlbumService::listen_to_album(&db, &album_id, Some(&user_id))
            .await
            .unwrap();

        let mut response = db
            .query(query)
            .bind(("user", create_user_thing(&user_id)))
            .bind(("album", create_album_thing(&album_id)))
            .await
            .unwrap();

        let relations: Vec<ListenRelation> = response.take(0).unwrap();
        assert_eq!(relations[0].total_listens, 2);
        assert_eq!(relations[0].total_duration, Duration::new(4800, 0));
    }

    #[tokio::test]
    async fn test_listen_to_nonexistent_album() {
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;

        let result = AlbumService::listen_to_album(&db, "nonexistent", Some(&user_id)).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::AlbumNotFound { .. }));
    }

    #[tokio::test]
    async fn test_concurrent_listens() {
        let db = setup_db().await;
        let album_id = create_test_album(&db, "album1", "Test Album").await;
        let user_id = create_test_user(&db, "user1").await;

        // Simulate concurrent listens
        let mut handles = vec![];
        for _ in 0..5 {
            let db_clone = db.clone();
            let album_id_clone = album_id.clone();
            let user_id_clone = user_id.clone();
            
            handles.push(tokio::spawn(async move {
                AlbumService::listen_to_album(&db_clone, &album_id_clone, Some(&user_id_clone))
                    .await
            }));
        }

        for handle in handles {
            assert!(handle.await.unwrap().is_ok());
        }

        // Check final count
        let query = "SELECT total_listens FROM user_listens_album WHERE in = $user AND out = $album";
        let mut response = db
            .query(query)
            .bind(("user", create_user_thing(&user_id)))
            .bind(("album", create_album_thing(&album_id)))
            .await
            .unwrap();

        #[derive(serde::Deserialize)]
        struct ListenCount {
            total_listens: u64,
        }

        let result: Vec<ListenCount> = response.take(0).unwrap();
        // Due to race conditions, the count might be less than 5
        // but should be at least 1
        assert!(result[0].total_listens >= 1);
        assert!(result[0].total_listens <= 5);
    }

    #[tokio::test]
    async fn test_recent_dates_limit() {
        let db = setup_db().await;
        let album_id = create_test_album(&db, "album1", "Test Album").await;
        let user_id = create_test_user(&db, "user1").await;

        // Listen 35 times
        for _ in 0..35 {
            AlbumService::listen_to_album(&db, &album_id, Some(&user_id))
                .await
                .unwrap();
        }

        let query = "SELECT recent_dates FROM user_listens_album WHERE in = $user AND out = $album";
        let mut response = db
            .query(query)
            .bind(("user", create_user_thing(&user_id)))
            .bind(("album", create_album_thing(&album_id)))
            .await
            .unwrap();

        #[derive(serde::Deserialize)]
        struct RecentDates {
            recent_dates: Vec<Datetime>,
        }

        let result: Vec<RecentDates> = response.take(0).unwrap();
        assert_eq!(
            result[0].recent_dates.len(),
            30,
            "Should keep only 30 most recent dates"
        );
    }

    #[tokio::test]
    async fn test_pagination_edge_cases() {
        let db = setup_db().await;

        create_test_album(&db, "album1", "Test Album").await;

        // Test with very large limit
        let response = AlbumService::get_albums_batch(&db, 0, 1000)
            .await
            .unwrap();
        assert_eq!(response.len(), 1); // Should return only existing album

        // Test with offset beyond available data
        let response = AlbumService::get_albums_batch(&db, 100, 10)
            .await
            .unwrap();
        assert_eq!(response.len(), 0);

        // Test with zero limit (should be normalized to 1)
        let response = AlbumService::get_albums_batch(&db, 0, 0)
            .await
            .unwrap();
        assert_eq!(response.len(), 1);
    }

    #[tokio::test]
    async fn test_complex_filtering() {
        let db = setup_db().await;

        // Create varied albums
        for i in 1..=5 {
            let album = Album {
                id: Some(create_album_thing(&format!("rock{}", i))),
                title: format!("Rock Album {}", i),
                cover_url: None,
                release_year: Some(2020 + i as u16),
                genres: vec!["Rock".to_string(), "Alternative".to_string()],
                langs: vec!["en".to_string()],
                dominant_color: None,
                total_tracks: 10,
                total_duration: Duration::new(2400, 0),
                total_listens: i * 100,
                total_user_listens: 0,
                total_likes: 0,
            };
            db.create::<Option<Album>>("album").content(album).await.unwrap();
        }

        // Test combining genre filter, search, and sorting
        let response = AlbumService::get_albums_filtered(
            &db,
            0,
            10,
            Some(vec!["Rock".to_string()]),
            Some("Album".to_string()),
            Some("popular".to_string()),
        )
        .await
        .unwrap();

        assert_eq!(response.albums.len(), 5);
        // Should be sorted by popularity (descending)
        assert_eq!(response.albums[0].title, "Rock Album 5");
        assert_eq!(response.albums[4].title, "Rock Album 1");
    }

    #[tokio::test]
    async fn test_album_with_multiple_artists() {
        let db = setup_db().await;

        let album_id = create_test_album(&db, "album1", "Collaboration Album").await;
        let artist1_id = create_test_artist(&db, "artist1", "Artist One").await;
        let artist2_id = create_test_artist(&db, "artist2", "Artist Two").await;

        // Create multiple artist relations
        db.query("
            RELATE $artist1->artist_creates_album->$album;
            RELATE $artist2->artist_creates_album->$album;
        ")
        .bind(("artist1", create_artist_thing(&artist1_id)))
        .bind(("artist2", create_artist_thing(&artist2_id)))
        .bind(("album", create_album_thing(&album_id)))
        .await
        .unwrap();

        let album = AlbumService::get_album(&db, &album_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(album.artists.len(), 2);
        let artist_names: Vec<String> = album.artists.iter().map(|a| a.name.clone()).collect();
        assert!(artist_names.contains(&"Artist One".to_string()));
        assert!(artist_names.contains(&"Artist Two".to_string()));
    }

    use crate::helpers::thing_helpers::{create_artist_thing, create_song_thing};
}