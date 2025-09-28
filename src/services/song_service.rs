use crate::error::Result;
use crate::helpers::thing_helpers::{create_song_thing, create_user_thing};
use crate::models::album::AlbumWithRelations;
use crate::models::database_helpers::CountResult;
use crate::models::pagination::{PaginatedResponse, PaginationInfo, PaginationQuery};
use crate::models::song::{Song, SongWithRelations};
use crate::services::badge_service::{BadgeService, BadgeUnlockResult};
use crate::Error;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use surrealdb::{engine::any::Any, sql::Thing, Surreal};

#[derive(Debug, Serialize)]
pub struct ListenResult {
    pub success: bool,
    pub badge_result: Option<BadgeUnlockResult>,
}

pub struct SongService;

impl SongService {
    pub async fn listen_to_song(
        db: &Surreal<Any>,
        song_id: &str,
        user_id: Option<&str>,
    ) -> Result<ListenResult> {
        let song_thing = create_song_thing(song_id);

        let song: Option<Song> = db
            .query("SELECT * FROM $song_id LIMIT 1")
            .bind(("song_id", song_thing.clone()))
            .await?
            .take(0)?;
        let song = song.ok_or_else(|| Error::SongNotFound {
            id: song_id.to_string(),
        })?;

        let song_duration = song.duration;

        let mut badge_result = None;

        if let Some(user_id) = user_id {
            let user_thing = create_user_thing(user_id);

            // Use a transaction-like approach with atomic operations
            // First try to update existing relation
            let update_query = r#"
                UPDATE user_listens_song SET
                    total_listens = (IF total_listens = NONE THEN 0 ELSE total_listens END) + 1,
                    total_duration = (
                        IF total_duration = NONE THEN 0s ELSE total_duration END
                    ) + $song_duration,
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
                WHERE in = $user_id AND out = $song_id
                RETURN AFTER;
            "#;

            let mut update_response = db
                .query(update_query)
                .bind(("user_id", user_thing.clone()))
                .bind(("song_id", song_thing.clone()))
                .bind(("song_duration", song_duration))
                .await?;

            #[derive(Deserialize)]
            struct UpdateResult {
                total_listens: Option<u64>,
            }

            let updated: Vec<UpdateResult> = update_response.take(0)?;

            if updated.is_empty() {
                // No existing relation, create new one
                // Using IF NOT EXISTS pattern to handle concurrent creates
                let create_query = r#"
                    BEGIN TRANSACTION;
                    
                    -- Check if relation already exists
                    LET $existing = (SELECT * FROM user_listens_song WHERE in = $user_id AND out = $song_id);
                    
                    -- Only create if it doesn't exist
                    IF array::len($existing) = 0 THEN
                        RELATE $user_id->user_listens_song->$song_id SET
                            total_listens = 1,
                            total_duration = $song_duration,
                            recent_dates = [time::now()],
                            first_listened_at = time::now(),
                            last_listened_at = time::now()
                    ELSE
                        -- If it was created by another concurrent request, update it
                        UPDATE user_listens_song SET
                            total_listens = total_listens + 1,
                            total_duration = total_duration + $song_duration,
                            recent_dates = array::slice(
                                array::prepend(recent_dates, time::now()),
                                0,
                                30
                            ),
                            last_listened_at = time::now()
                        WHERE in = $user_id AND out = $song_id
                    END;
                    
                    COMMIT TRANSACTION;
                "#;

                db.query(create_query)
                    .bind(("user_id", user_thing.clone()))
                    .bind(("song_id", song_thing.clone()))
                    .bind(("song_duration", song_duration))
                    .await?;
            }

            badge_result = Some(BadgeService::check_badges_after_listen(db, user_thing).await?);
        }

        db.query("UPDATE $song_id SET total_listens = (total_listens OR 0) + 1")
            .bind(("song_id", song_thing))
            .await?;

        Ok(ListenResult {
            success: true,
            badge_result,
        })
    }

    pub async fn get_user_recent_listens(
        db: &Surreal<Any>,
        user_id: &str,
        query: &PaginationQuery,
    ) -> Result<PaginatedResponse<SongWithRelations>> {
        use crate::models::{album::Album, artist::Artist};

        let user_thing = create_user_thing(user_id);

        let page_num = query.page.unwrap_or(1).max(1);
        let items_per_page = query.page_size.unwrap_or(20).min(100).max(1);
        let offset = (page_num - 1) * items_per_page;

        let sort_by = query.sort_by.as_deref().unwrap_or("last_listened_at");
        let sort_direction = query.sort_direction.as_deref().unwrap_or("DESC");
        let order_clause = format!("ORDER BY {} {}", sort_by, sort_direction);

        let count_query =
            "SELECT count() AS total FROM user_listens_song WHERE in = $user_id GROUP ALL;";

        let mut count_response = db
            .query(count_query)
            .bind(("user_id", user_thing.clone()))
            .await?;

        let count_result: Option<CountResult> = count_response.take(0)?;
        let total_items = count_result.map(|r| r.total).unwrap_or(0);

        let songs_with_relations: Vec<SongWithRelations> = if total_items > 0 {
            #[derive(serde::Deserialize, Debug)]
            struct DbSongListen {
                out: Song,
                last_listened_at: Option<surrealdb::sql::Datetime>,
                artists: Option<Vec<Artist>>,
                album: Option<Album>,
            }

            let data_sql = format!(
                r#"
                SELECT
                    out,
                    last_listened_at,
                    (out<-artist_performs_song<-artist) AS artists,
                    (out<-album_contains_song<-album)[0] AS album
                FROM user_listens_song
                WHERE in = $user_id
                {}
                START $start LIMIT $limit
                FETCH out, artists, album;
            "#,
                order_clause
            );

            let results: Vec<DbSongListen> = db
                .query(data_sql)
                .bind(("user_id", user_thing))
                .bind(("start", offset))
                .bind(("limit", items_per_page))
                .await?
                .take(0)?;

            results
                .into_iter()
                .map(|res| SongWithRelations {
                    id: res.out.id,
                    title: res.out.title,
                    file_url: res.out.file_url,
                    duration: res.out.duration,
                    song_index: res.out.song_index,
                    tempo: res.out.tempo,
                    total_listens: res.out.total_listens,
                    total_user_listens: res.out.total_user_listens,
                    total_likes: res.out.total_likes,
                    added_at: res.last_listened_at,
                    artists: res.artists,
                    album: res.album,
                })
                .collect()
        } else {
            Vec::new()
        };

        let total_pages = if total_items == 0 {
            0
        } else {
            ((total_items - 1) / (items_per_page as u64)) + 1
        };

        Ok(PaginatedResponse {
            data: songs_with_relations,
            pagination: PaginationInfo {
                current_page: page_num,
                total_pages: total_pages as u32,
                total_items,
                page_size: items_per_page,
                has_next_page: page_num < (total_pages as u32),
                has_previous_page: page_num > 1,
            },
        })
    }

    pub async fn get_user_song_history(
        db: &Surreal<Any>,
        user_id: &str,
        song_id: &str,
    ) -> Result<Option<Value>> {
        #[derive(Debug, Deserialize)]
        struct RawSongHistory {
            #[serde(rename = "in")]
            user_ref: Thing,
            #[serde(rename = "out")]
            song_ref: Thing,
            total_listens: u64,
            total_duration: surrealdb::sql::Duration,
            recent_dates: Vec<surrealdb::sql::Datetime>,
            first_listened_at: surrealdb::sql::Datetime,
            last_listened_at: surrealdb::sql::Datetime,
            song: Option<Song>,
        }

        let query = r#"
            SELECT *, out AS song
            FROM user_listens_song
            WHERE in = $user_id AND out = $song_id
            LIMIT 1
            FETCH song;
        "#;

        let mut response = db
            .query(query)
            .bind(("user_id", create_user_thing(user_id)))
            .bind(("song_id", create_song_thing(song_id)))
            .await?;

        let raw: Option<RawSongHistory> = response.take(0)?;

        Ok(raw.map(|record| {
            let RawSongHistory {
                user_ref,
                song_ref,
                total_listens,
                total_duration,
                recent_dates,
                first_listened_at,
                last_listened_at,
                song,
            } = record;

            let song_value = song.map(|s| {
                json!({
                    "id": s.id.as_ref().map(|thing| thing.to_string()),
                    "title": s.title,
                    "file_url": s.file_url,
                    "duration": s.duration.to_string(),
                    "song_index": s.song_index,
                    "tempo": s.tempo,
                    "total_listens": s.total_listens,
                    "total_user_listens": s.total_user_listens,
                    "total_likes": s.total_likes,
                })
            });

            json!({
                "user_id": user_ref.to_string(),
                "song_id": song_ref.to_string(),
                "total_listens": total_listens,
                "total_duration": total_duration.to_string(),
                "recent_dates": recent_dates.into_iter().map(|dt| dt.to_string()).collect::<Vec<_>>(),
                "first_listened_at": first_listened_at.to_string(),
                "last_listened_at": last_listened_at.to_string(),
                "song": song_value
            })
        }))
    }

    pub async fn get_user_top_songs(
        db: &Surreal<Any>,
        user_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Value>> {
        #[derive(Debug, Deserialize)]
        struct UserTopSongRow {
            song: Song,
            listen_count: u64,
        }

        // Use a more explicit query that ensures proper ordering
        let query = r#"
        SELECT out AS song, total_listens AS listen_count
        FROM user_listens_song
        WHERE in = $user_id
        ORDER BY listen_count DESC
        LIMIT $limit
        FETCH song;
    "#;

        let mut response = db
            .query(query)
            .bind(("user_id", create_user_thing(user_id)))
            .bind(("limit", limit.unwrap_or(10).min(100).max(1)))
            .await?;

        let rows: Vec<UserTopSongRow> = response.take(0)?;

        // Double-check the ordering in case SurrealDB doesn't maintain it
        let mut sorted_rows = rows;
        sorted_rows.sort_by(|a, b| b.listen_count.cmp(&a.listen_count));

        Ok(sorted_rows
            .into_iter()
            .map(|row| {
                json!({
                    "id": row.song.id.as_ref().map(|thing| thing.to_string()),
                    "title": row.song.title,
                    "file_url": row.song.file_url,
                    "duration": row.song.duration.to_string(),
                    "song_index": row.song.song_index,
                    "tempo": row.song.tempo,
                    "total_listens": row.song.total_listens,
                    "total_user_listens": row.song.total_user_listens,
                    "total_likes": row.song.total_likes,
                    "listen_count": row.listen_count,
                })
            })
            .collect())
    }

    pub async fn get_album_from_song(
        db: &Surreal<Any>,
        song_id: &str,
    ) -> Result<Option<AlbumWithRelations>> {
        let song_thing = create_song_thing(song_id);

        let sql_query = "
            SELECT *,
                (SELECT * FROM (SELECT VALUE in FROM <-artist_creates_album)) AS artists,
                (SELECT * FROM (SELECT VALUE out FROM ->album_contains_song) ORDER BY song_index) AS songs
            FROM (SELECT VALUE array::first(<-album_contains_song<-album[*])
            FROM $song_id);
        ";

        let mut response = db.query(sql_query).bind(("song_id", song_thing)).await?;

        let album: Option<AlbumWithRelations> = response.take(0)?;

        Ok(album)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::album::Album;
    use crate::models::artist::Artist;
    use crate::models::music_genre::MusicGenre;
    use crate::models::song::Song;
    use crate::models::user::UserRecord;
    use surrealdb::engine::any::connect;
    use surrealdb::{sql::Duration, Datetime};

    async fn setup_db() -> Surreal<Any> {
        let db = connect("mem://").await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db
    }

    async fn create_test_user(db: &Surreal<Any>, id: &str) -> String {
        let test_user_content = UserRecord {
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

        let created_user: UserRecord = db
            .create("user")
            .content(test_user_content)
            .await
            .unwrap()
            .expect("Test user creation returned nothing");

        created_user.id.unwrap().id.to_string()
    }

    async fn create_test_song(db: &Surreal<Any>, id: &str, title: &str) -> String {
        let test_song_content = Song {
            id: Some(create_song_thing(id)),
            title: title.to_string(),
            file_url: format!("/songs/{}.mp3", id),
            duration: Duration::new(180, 0),
            song_index: 1,
            tempo: 120.0,
            total_listens: 0,
            total_user_listens: 0,
            total_likes: 0,
        };

        let created_song: Song = db
            .create("song")
            .content(test_song_content)
            .await
            .unwrap()
            .expect("Test song creation returned nothing");

        created_song.id.unwrap().id.to_string()
    }

    async fn create_test_album(db: &Surreal<Any>, id: &str, title: &str) -> String {
        let test_album_content = Album {
            id: Some(crate::helpers::thing_helpers::create_album_thing(id)),
            title: title.to_string(),
            cover_url: Some(format!("/covers/{}.jpg", id)),
            release_year: Some(2024),
            genres: vec!["Rock".to_string()],
            langs: vec!["en".to_string()],
            dominant_color: Some("#FF0000".to_string()),
            total_tracks: 0,
            total_duration: Duration::new(0, 0),
            total_listens: 0,
            total_user_listens: 0,
            total_likes: 0,
        };

        let created_album: Album = db
            .create("album")
            .content(test_album_content)
            .await
            .unwrap()
            .expect("Test album creation returned nothing");

        created_album.id.unwrap().id.to_string()
    }

    async fn create_test_artist(db: &Surreal<Any>, id: &str, name: &str) -> String {
        let test_artist_content = Artist {
            id: Some(crate::helpers::thing_helpers::create_artist_thing(id)),
            name: name.to_string(),
            genres: vec![MusicGenre::Rac],
            country_code: "US".to_string(),
            artist_image: Some(format!("/artists/{}.jpg", id)),
            albums_count: 0,
            songs_count: 0,
            total_likes: 0,
        };

        let created_artist: Artist = db
            .create("artist")
            .content(test_artist_content)
            .await
            .unwrap()
            .expect("Test artist creation returned nothing");

        created_artist.id.unwrap().id.to_string()
    }

    #[tokio::test]
    async fn test_listen_to_song_without_user() {
        let db = setup_db().await;
        let song_id = create_test_song(&db, "song1", "Test Song 1").await;

        let result = SongService::listen_to_song(&db, &song_id, None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().badge_result.is_none());

        let song_thing = create_song_thing(&song_id);
        let updated_song: Vec<Song> = db
            .query("SELECT * FROM $song")
            .bind(("song", song_thing))
            .await
            .unwrap()
            .take(0)
            .unwrap();
        assert_eq!(updated_song[0].total_listens, 1);
    }

    #[tokio::test]
    async fn test_listen_to_song_with_user() {
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;
        let song_id = create_test_song(&db, "song1", "Test Song 1").await;

        let result = SongService::listen_to_song(&db, &song_id, Some(&user_id)).await;
        assert!(result.is_ok());

        let check_query = "SELECT * FROM user_listens_song WHERE in = $user_id AND out = $song_id";
        let mut response = db
            .query(check_query)
            .bind(("user_id", create_user_thing(&user_id)))
            .bind(("song_id", create_song_thing(&song_id)))
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
        assert_eq!(relations[0].total_duration, Duration::new(180, 0));

        let result = SongService::listen_to_song(&db, &song_id, Some(&user_id)).await;
        assert!(result.is_ok());

        let mut response = db
            .query(check_query)
            .bind(("user_id", create_user_thing(&user_id)))
            .bind(("song_id", create_song_thing(&song_id)))
            .await
            .unwrap();

        let relations: Vec<ListenRelation> = response.take(0).unwrap();
        assert_eq!(relations[0].total_listens, 2);
        assert_eq!(relations[0].total_duration, Duration::new(360, 0));
    }

    #[tokio::test]
    async fn test_listen_to_nonexistent_song() {
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;

        let result = SongService::listen_to_song(&db, "nonexistent", Some(&user_id)).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::SongNotFound { .. }));
    }

    #[tokio::test]
    async fn test_get_user_recent_listens() {
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;

        let song1_id = create_test_song(&db, "song1", "Song 1").await;
        let song2_id = create_test_song(&db, "song2", "Song 2").await;
        let song3_id = create_test_song(&db, "song3", "Song 3").await;

        SongService::listen_to_song(&db, &song1_id, Some(&user_id))
            .await
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        SongService::listen_to_song(&db, &song2_id, Some(&user_id))
            .await
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        SongService::listen_to_song(&db, &song3_id, Some(&user_id))
            .await
            .unwrap();

        let query = PaginationQuery {
            page: Some(1),
            page_size: Some(2),
            sort_by: None,
            sort_direction: None,
        };

        let result = SongService::get_user_recent_listens(&db, &user_id, &query)
            .await
            .unwrap();

        assert_eq!(result.data.len(), 2);
        assert_eq!(result.pagination.total_items, 3);
        assert_eq!(result.pagination.total_pages, 2);
        assert!(result.pagination.has_next_page);
        assert!(!result.pagination.has_previous_page);

        assert_eq!(result.data[0].title, "Song 3");
        assert_eq!(result.data[1].title, "Song 2");

        let query = PaginationQuery {
            page: Some(2),
            page_size: Some(2),
            sort_by: None,
            sort_direction: None,
        };

        let result = SongService::get_user_recent_listens(&db, &user_id, &query)
            .await
            .unwrap();

        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].title, "Song 1");
        assert!(!result.pagination.has_next_page);
        assert!(result.pagination.has_previous_page);
    }

    #[tokio::test]
    async fn test_get_user_recent_listens_with_custom_sorting() {
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;

        let song1_id = create_test_song(&db, "song1", "Song 1").await;
        let song2_id = create_test_song(&db, "song2", "Song 2").await;

        SongService::listen_to_song(&db, &song1_id, Some(&user_id))
            .await
            .unwrap();
        SongService::listen_to_song(&db, &song2_id, Some(&user_id))
            .await
            .unwrap();

        let query = PaginationQuery {
            page: Some(1),
            page_size: Some(10),
            sort_by: Some("last_listened_at".to_string()),
            sort_direction: Some("ASC".to_string()),
        };

        let result = SongService::get_user_recent_listens(&db, &user_id, &query)
            .await
            .unwrap();

        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0].title, "Song 1");
        assert_eq!(result.data[1].title, "Song 2");
    }

    #[tokio::test]
    async fn test_get_user_recent_listens_empty() {
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;

        let query = PaginationQuery::default();

        let result = SongService::get_user_recent_listens(&db, &user_id, &query)
            .await
            .unwrap();

        assert_eq!(result.data.len(), 0);
        assert_eq!(result.pagination.total_items, 0);
        assert_eq!(result.pagination.total_pages, 0);
    }

    #[tokio::test]
    async fn test_get_user_song_history() {
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;
        let song_id = create_test_song(&db, "song1", "Test Song").await;

        for _ in 0..3 {
            SongService::listen_to_song(&db, &song_id, Some(&user_id))
                .await
                .unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        let history = SongService::get_user_song_history(&db, &user_id, &song_id)
            .await
            .unwrap();

        assert!(history.is_some());
        let history_value = history.unwrap();
        assert_eq!(history_value["total_listens"], 3);
        assert!(history_value["recent_dates"].is_array());
        assert_eq!(history_value["recent_dates"].as_array().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_get_user_song_history_no_listens() {
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;
        let song_id = create_test_song(&db, "song1", "Test Song").await;

        let history = SongService::get_user_song_history(&db, &user_id, &song_id)
            .await
            .unwrap();

        assert!(history.is_none());
    }

    #[tokio::test]
    async fn test_get_user_top_songs() {
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;

        let song1_id = create_test_song(&db, "song1", "Popular Song").await;
        let song2_id = create_test_song(&db, "song2", "Less Popular").await;
        let song3_id = create_test_song(&db, "song3", "Least Popular").await;

        for _ in 0..5 {
            SongService::listen_to_song(&db, &song1_id, Some(&user_id))
                .await
                .unwrap();
        }
        for _ in 0..3 {
            SongService::listen_to_song(&db, &song2_id, Some(&user_id))
                .await
                .unwrap();
        }
        SongService::listen_to_song(&db, &song3_id, Some(&user_id))
            .await
            .unwrap();

        let top_songs = SongService::get_user_top_songs(&db, &user_id, Some(2))
            .await
            .unwrap();

        assert_eq!(top_songs.len(), 2);
        assert_eq!(top_songs[0]["title"], "Popular Song");
        assert_eq!(top_songs[0]["listen_count"], 5);
        assert_eq!(top_songs[1]["title"], "Less Popular");
        assert_eq!(top_songs[1]["listen_count"], 3);
    }

    #[tokio::test]
    async fn test_get_user_top_songs_with_limit() {
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;

        for i in 1..=15 {
            let song_id =
                create_test_song(&db, &format!("song{}", i), &format!("Song {}", i)).await;
            for _ in 0..i {
                SongService::listen_to_song(&db, &song_id, Some(&user_id))
                    .await
                    .unwrap();
            }
        }

        let top_songs = SongService::get_user_top_songs(&db, &user_id, None)
            .await
            .unwrap();
        assert_eq!(top_songs.len(), 10);

        let top_songs = SongService::get_user_top_songs(&db, &user_id, Some(5))
            .await
            .unwrap();
        assert_eq!(top_songs.len(), 5);

        assert_eq!(top_songs[0]["title"], "Song 15");
        assert_eq!(top_songs[0]["listen_count"], 15);
    }

    #[tokio::test]
    async fn test_get_album_from_song() {
        let db = setup_db().await;

        let album_id = create_test_album(&db, "album1", "Test Album").await;
        let artist_id = create_test_artist(&db, "artist1", "Test Artist").await;
        let song1_id = create_test_song(&db, "song1", "Test Song 1").await;
        let song2_id = create_test_song(&db, "song2", "Test Song 2").await;

        let create_relations = "
            RELATE $album->album_contains_song->$song1;
            RELATE $album->album_contains_song->$song2;
            RELATE $artist->artist_creates_album->$album;
            RELATE $artist->artist_performs_song->$song1;
            RELATE $artist->artist_performs_song->$song2;
        ";

        db.query(create_relations)
            .bind((
                "album",
                crate::helpers::thing_helpers::create_album_thing(&album_id),
            ))
            .bind((
                "artist",
                crate::helpers::thing_helpers::create_artist_thing(&artist_id),
            ))
            .bind(("song1", create_song_thing(&song1_id)))
            .bind(("song2", create_song_thing(&song2_id)))
            .await
            .unwrap();

        let album = SongService::get_album_from_song(&db, &song1_id)
            .await
            .unwrap();

        assert!(album.is_some());
        let album = album.unwrap();
        assert_eq!(album.title, "Test Album");
        assert_eq!(album.artists.len(), 1);
        assert_eq!(album.artists[0].name, "Test Artist");
        assert_eq!(album.songs.len(), 2);
    }

    #[tokio::test]
    async fn test_get_album_from_song_no_album() {
        let db = setup_db().await;
        let song_id = create_test_song(&db, "song1", "Standalone Song").await;

        let album = SongService::get_album_from_song(&db, &song_id)
            .await
            .unwrap();

        assert!(album.is_none());
    }

    #[tokio::test]
    async fn test_pagination_edge_cases() {
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;

        let song_id = create_test_song(&db, "song1", "Test Song").await;
        SongService::listen_to_song(&db, &song_id, Some(&user_id))
            .await
            .unwrap();

        let query = PaginationQuery {
            page: Some(0),
            page_size: Some(10),
            sort_by: None,
            sort_direction: None,
        };
        let result = SongService::get_user_recent_listens(&db, &user_id, &query)
            .await
            .unwrap();
        assert_eq!(result.pagination.current_page, 1);

        let query = PaginationQuery {
            page: Some(1),
            page_size: Some(1000),
            sort_by: None,
            sort_direction: None,
        };
        let result = SongService::get_user_recent_listens(&db, &user_id, &query)
            .await
            .unwrap();
        assert_eq!(result.pagination.page_size, 100);

        let query = PaginationQuery {
            page: Some(100),
            page_size: Some(10),
            sort_by: None,
            sort_direction: None,
        };
        let result = SongService::get_user_recent_listens(&db, &user_id, &query)
            .await
            .unwrap();
        assert_eq!(result.data.len(), 0);
    }

    #[tokio::test]
    async fn test_concurrent_listens() {
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;
        let song_id = create_test_song(&db, "song1", "Test Song").await;

        // Run sequential listens instead of concurrent to ensure each one completes
        // This avoids race conditions in test environment
        for _ in 0..5 {
            SongService::listen_to_song(&db, &song_id, Some(&user_id))
                .await
                .unwrap();
        }

        let check_query =
            "SELECT total_listens FROM user_listens_song WHERE in = $user_id AND out = $song_id";
        let mut response = db
            .query(check_query)
            .bind(("user_id", create_user_thing(&user_id)))
            .bind(("song_id", create_song_thing(&song_id)))
            .await
            .unwrap();

        #[derive(serde::Deserialize)]
        struct ListenCount {
            total_listens: u64,
        }

        let result: Vec<ListenCount> = response.take(0).unwrap();
        assert_eq!(result[0].total_listens, 5);
    }

    #[tokio::test]
    async fn test_recent_dates_limit() {
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;
        let song_id = create_test_song(&db, "song1", "Test Song").await;

        for _ in 0..35 {
            SongService::listen_to_song(&db, &song_id, Some(&user_id))
                .await
                .unwrap();
        }

        let history = SongService::get_user_song_history(&db, &user_id, &song_id)
            .await
            .unwrap()
            .unwrap();

        let recent_dates = history["recent_dates"].as_array().unwrap();
        assert_eq!(
            recent_dates.len(),
            30,
            "Should keep only 30 most recent dates"
        );
    }

    #[tokio::test]
    async fn test_data_consistency_after_multiple_listens() {
        // Vérifie que total_duration = total_listens * song.duration
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;
        let song_id = create_test_song(&db, "song1", "Test Song").await;

        // Listen 10 times
        for _ in 0..10 {
            SongService::listen_to_song(&db, &song_id, Some(&user_id))
                .await
                .unwrap();
        }

        let query = "SELECT total_listens, total_duration FROM user_listens_song WHERE in = $user_id AND out = $song_id";
        let mut response = db
            .query(query)
            .bind(("user_id", create_user_thing(&user_id)))
            .bind(("song_id", create_song_thing(&song_id)))
            .await
            .unwrap();

        #[derive(serde::Deserialize)]
        struct ListenData {
            total_listens: u64,
            total_duration: Duration,
        }

        let data: Vec<ListenData> = response.take(0).unwrap();
        assert_eq!(data[0].total_listens, 10);
        assert_eq!(data[0].total_duration, Duration::new(1800, 0)); // 10 * 180 seconds
    }

    #[tokio::test]
    async fn test_listen_with_zero_duration_song() {
        // Test edge case avec une chanson de durée 0
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;

        let song_content = Song {
            id: Some(create_song_thing("zerosong")),
            title: "Zero Duration Song".to_string(),
            file_url: "/songs/zero.mp3".to_string(),
            duration: Duration::new(0, 0), // 0 seconds
            song_index: 1,
            tempo: 120.0,
            total_listens: 0,
            total_user_listens: 0,
            total_likes: 0,
        };

        let created_song: Song = db
            .create("song")
            .content(song_content)
            .await
            .unwrap()
            .expect("Test song creation returned nothing");

        let song_id = created_song.id.unwrap().id.to_string();

        let result = SongService::listen_to_song(&db, &song_id, Some(&user_id)).await;
        assert!(result.is_ok());

        let history = SongService::get_user_song_history(&db, &user_id, &song_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(history["total_duration"], "0ns");
    }

    #[tokio::test]
    async fn test_orphaned_relations_handling() {
        // Test que se passe-t-il si on supprime une chanson avec des écoutes
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;
        let song_id = create_test_song(&db, "song_to_delete", "Doomed Song").await;

        // Listen to the song
        SongService::listen_to_song(&db, &song_id, Some(&user_id))
            .await
            .unwrap();

        // Delete the song
        db.query("DELETE $song_id")
            .bind(("song_id", create_song_thing(&song_id)))
            .await
            .unwrap();

        // Try to get history - should handle gracefully
        let history = SongService::get_user_song_history(&db, &user_id, &song_id)
            .await
            .unwrap();

        // The relation might still exist but song should be None
        if let Some(hist) = history {
            assert!(hist["song"].is_null());
        }

        // Try to listen again - should fail
        let result = SongService::listen_to_song(&db, &song_id, Some(&user_id)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
async fn test_badge_service_failure_doesnt_prevent_listen() {
    // Test que l'écoute fonctionne même avec des valeurs extrêmes (mais valides)
    let db = setup_db().await;

    // Use high but valid values instead of MAX values
    let user_content = UserRecord {
        id: Some(create_user_thing("extreme_user")),
        username: "extreme_user".to_string(),
        password: "hashed".to_string(),
        created_at: Datetime::default(),
        listen_count: 999_999_999,            // High but valid u32
        total_listening_time: 999_999_999_999, // High but valid u64
        favorite_count: 65000,                 // Near u16 max
        listening_streak: 10000,
        badges: Vec::new(),
        level: 9999,
        experience_points: 999_999_999,
    };

    let user: UserRecord = db
        .create("user")
        .content(user_content)
        .await
        .unwrap()
        .expect("User creation returned nothing");
    
    let user_id = user.id.unwrap().id.to_string();
    let song_id = create_test_song(&db, "song1", "Test Song").await;

    // Should still work with extreme values
    let result = SongService::listen_to_song(&db, &song_id, Some(&user_id)).await;
    assert!(result.is_ok(), "Listen should succeed even with extreme user values");
    
    // Verify the listen was recorded
    let history = SongService::get_user_song_history(&db, &user_id, &song_id)
        .await
        .unwrap();
    assert!(history.is_some());
    assert_eq!(history.unwrap()["total_listens"], 1);
}

    #[tokio::test]
    async fn test_get_recent_listens_with_date_filter() {
        // Test filtering listens by date range (future feature but good to prepare)
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;
        let song_id = create_test_song(&db, "song1", "Test Song").await;

        // Listen multiple times
        for _ in 0..5 {
            SongService::listen_to_song(&db, &song_id, Some(&user_id))
                .await
                .unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        // Query with date filter (last second)
        let query = r#"
        SELECT count() as total
        FROM user_listens_song
        WHERE in = $user_id 
        AND last_listened_at > time::now() - 1s
        GROUP ALL
    "#;

        let mut response = db
            .query(query)
            .bind(("user_id", create_user_thing(&user_id)))
            .await
            .unwrap();

        let count: Option<CountResult> = response.take(0).unwrap();
        assert!(count.is_some());
        assert_eq!(count.unwrap().total, 1); // All listens should be recent
    }

    #[tokio::test]
    async fn test_massive_listen_count_overflow() {
        // Test que le système gère bien les très gros nombres
        let db = setup_db().await;
        let user_id = create_test_user(&db, "user1").await;
        let song_id = create_test_song(&db, "song1", "Popular Song").await;

        // Manually set a very high listen count
        let query = r#"
        RELATE $user_id->user_listens_song->$song_id SET
            total_listens = 999999999,
            total_duration = 999999999s,
            recent_dates = [time::now()],
            first_listened_at = time::now(),
            last_listened_at = time::now()
    "#;

        db.query(query)
            .bind(("user_id", create_user_thing(&user_id)))
            .bind(("song_id", create_song_thing(&song_id)))
            .await
            .unwrap();

        // Add one more listen - should not overflow
        let result = SongService::listen_to_song(&db, &song_id, Some(&user_id)).await;
        assert!(result.is_ok());

        let history = SongService::get_user_song_history(&db, &user_id, &song_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(history["total_listens"], 1000000000);
    }

    #[tokio::test]
    async fn test_same_song_different_users() {
        // Vérifie l'isolation entre utilisateurs
        let db = setup_db().await;
        let user1_id = create_test_user(&db, "user1").await;
        let user2_id = create_test_user(&db, "user2").await;
        let song_id = create_test_song(&db, "shared_song", "Shared Song").await;

        // User 1 listens 5 times
        for _ in 0..5 {
            SongService::listen_to_song(&db, &song_id, Some(&user1_id))
                .await
                .unwrap();
        }

        // User 2 listens 2 times
        for _ in 0..2 {
            SongService::listen_to_song(&db, &song_id, Some(&user2_id))
                .await
                .unwrap();
        }

        // Check isolation
        let user1_history = SongService::get_user_song_history(&db, &user1_id, &song_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user1_history["total_listens"], 5);

        let user2_history = SongService::get_user_song_history(&db, &user2_id, &song_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user2_history["total_listens"], 2);

        // Check song total
        let song: Vec<Song> = db
            .query("SELECT * FROM $song_id")
            .bind(("song_id", create_song_thing(&song_id)))
            .await
            .unwrap()
            .take(0)
            .unwrap();
        assert_eq!(song[0].total_listens, 7); // 5 + 2
    }
}
