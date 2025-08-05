use crate::error::Result;
use crate::helpers::song_helpers::song_exists;
use crate::helpers::thing_helpers::{create_song_thing, create_user_thing};
use crate::models::album::AlbumWithRelations;
use crate::models::pagination::{PaginatedResponse, PaginationInfo, PaginationQuery};
use crate::models::song::SongWithRelations;
use crate::Error;
use surrealdb::{engine::any::Any, sql::Thing, Surreal};

#[derive(serde::Deserialize)]
struct CountResult {
    total: u64,
}

#[derive(serde::Deserialize)]
struct RelationId {
    id: Thing,
}

pub struct SongService;

impl SongService {
    pub async fn listen_to_song(
        db: &Surreal<Any>,
        song_id: &str,
        user_id: Option<&str>,
    ) -> Result<bool> {
        let song_thing = create_song_thing(song_id);

        if !song_exists(db, &song_id).await? {
            return Err(Error::SongNotFound {
                id: song_id.to_string(),
            });
        }

        if let Some(user_id) = user_id {
            let user_thing = create_user_thing(user_id);

            let check_query =
                "SELECT count() as total FROM user_listens_song WHERE in = $user_id AND out = $song_id GROUP ALL";

            let mut check_response = db
                .query(check_query)
                .bind(("user_id", user_thing.clone()))
                .bind(("song_id", song_thing.clone()))
                .await?;

            let count_result: Option<CountResult> = check_response.take(0)?;
            let exists = count_result.map(|r| r.total > 0).unwrap_or(false);

            // If exists => we create a new entry, else we update the existing one
            if exists {
                let get_id_query = "SELECT id FROM user_listens_song WHERE in = $user_id AND out = $song_id LIMIT 1";
                let mut id_response = db
                    .query(get_id_query)
                    .bind(("user_id", user_thing.clone()))
                    .bind(("song_id", song_thing.clone()))
                    .await?;

                let existing_relation: Option<RelationId> = id_response.take(0)?;

                if let Some(relation) = existing_relation {
                    let relation_id_str = relation.id.to_string();

                    let update_query = format!(
                        "LET $song_duration = (SELECT VALUE duration FROM $song_id LIMIT 1)[0] OR 0s;
                         UPDATE {} SET
                             total_listens += 1,
                             total_duration = total_duration + $song_duration,
                             recent_dates = array::slice(array::prepend(recent_dates, time::now()), 0, 30),
                             last_listened_at = time::now();",
                        relation_id_str
                    );

                    let _ = db
                        .query(update_query)
                        .bind(("song_id", song_thing.clone()))
                        .await?;

                    //let _update_result: Vec<serde_json::Value> = update_response.take(0)?;
                } else {
                    return Err(Error::DbError("No existing relation found".to_string()));
                }
            } else {
                let create_query = "
                    LET $song_duration = (SELECT VALUE duration FROM $song_id LIMIT 1)[0] OR 0s;
                    CREATE user_listens_song CONTENT {
                        in: $user_id,
                        out: $song_id,
                        total_listens: 1,
                        total_duration: $song_duration,
                        recent_dates: [time::now()],
                        first_listened_at: time::now(),
                        last_listened_at: time::now()
                    };
                ";

                let mut create_response = db
                    .query(create_query)
                    .bind(("user_id", user_thing))
                    .bind(("song_id", song_thing.clone()))
                    .await?;

                let create_result: Vec<serde_json::Value> = create_response.take(0)?;

                if create_result.is_empty() {
                    return Err(Error::DbError(
                        "Failed to create listen relation".to_string(),
                    ));
                }
            }
        }

        let update_query = "UPDATE $song_id SET total_listens = (total_listens OR 0) + 1";
        let _update_response = db.query(update_query).bind(("song_id", song_thing)).await?;

        Ok(true)
    }

    pub async fn get_user_recent_listens(
        db: &Surreal<Any>,
        user_id: &str,
        query: &PaginationQuery,
    ) -> Result<PaginatedResponse<SongWithRelations>> {
        use crate::models::{album::Album, artist::Artist, song::Song};

        let user_thing = create_user_thing(user_id);

        let page_num = query.page.unwrap_or(1);
        let items_per_page = query.page_size.unwrap_or(20);
        let offset = (page_num - 1) * items_per_page;

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
                _last_listened_at: Option<surrealdb::sql::Datetime>,
                added_at: Option<surrealdb::sql::Datetime>,
                artists: Option<Vec<Artist>>,
                album: Option<Album>,
            }

            let data_sql = r#"
                SELECT out, last_listened_at, recent_dates[0] as added_at, (out<-artist_performs_song<-artist) AS artists, (out<-album_contains_song<-album)[0] AS album
                FROM user_listens_song
                WHERE in = $user_id
                ORDER BY last_listened_at DESC
                START $start LIMIT $limit
                FETCH out, artists, album;
            "#;

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
                    added_at: res.added_at,
                    artists: res.artists,
                    album: res.album,
                })
                .collect()
        } else {
            Vec::new()
        };

        let total_pages = (total_items as f64 / items_per_page as f64).ceil() as u32;

        Ok(PaginatedResponse {
            data: songs_with_relations,
            pagination: PaginationInfo {
                current_page: page_num,
                total_pages,
                total_items,
                page_size: items_per_page,
                has_next_page: page_num < total_pages,
                has_previous_page: page_num > 1,
            },
        })
    }

    /// Récupérer l'historique détaillé d'une chanson pour un utilisateur
    pub async fn get_user_song_history(
        db: &Surreal<Any>,
        user_id: &str,
        song_id: &str,
    ) -> Result<Option<serde_json::Value>> {
        let user_thing = create_user_thing(user_id);
        let song_thing = create_song_thing(song_id);

        let query = "
        SELECT
            *,
            out.*,
            recent_dates
        FROM user_listens_song
        WHERE in = $user_id AND out = $song_id;
    ";

        let mut response = db
            .query(query)
            .bind(("user_id", user_thing))
            .bind(("song_id", song_thing))
            .await?;

        let song_history: Option<serde_json::Value> = response.take(0)?;
        Ok(song_history)
    }

    /// Récupérer les chansons les plus écoutées par un utilisateur
    pub async fn get_user_top_songs(
        db: &Surreal<Any>,
        user_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<serde_json::Value>> {
        let user_thing = create_user_thing(user_id);
        let limit_value = limit.unwrap_or(10);

        let query = "
        SELECT out.*, count() AS listen_count
        FROM user_listens_song
        WHERE in = $user_id
        GROUP BY out
        ORDER BY listen_count DESC
        LIMIT $limit;
    ";

        let mut response = db
            .query(query)
            .bind(("user_id", user_thing))
            .bind(("limit", limit_value))
            .await?;

        let top_songs: Vec<serde_json::Value> = response.take(0)?;
        Ok(top_songs)
    }

    pub async fn get_album_from_song(
        db: &Surreal<Any>,
        song_id: &str,
    ) -> Result<Option<AlbumWithRelations>> {
        let song_thing = create_song_thing(song_id);

        let sql_query = "SELECT *,
        (SELECT * FROM (SELECT VALUE in FROM <-artist_creates_album)) AS artists,
        (SELECT * FROM (SELECT VALUE out FROM ->album_contains_song) ORDER BY song_index) AS songs
        FROM (SELECT VALUE array::first(<-album_contains_song<-album[*]) 
        FROM $song_id);";

        let mut response = db.query(sql_query).bind(("song_id", song_thing)).await?;

        let album: Option<AlbumWithRelations> = response.take(0)?;

        Ok(album)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::user::UserProfile;

    use super::*;
    use surrealdb::engine::any::connect;
    use surrealdb::{sql::Duration, Datetime};

    async fn setup_db() -> (Surreal<Any>, String, String) {
        let db = connect("mem://").await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();

        let test_user_content = UserProfile {
            id: Some(create_user_thing("test")),
            username: "test".to_string(),
            created_at: Datetime::default(),
            listen_count: 0,
            total_listening_time: 0,
            favorite_count: 0,
            listening_streak: 0,
            badges: Vec::new(),
            level: 0,
            experience_points: 0,
        };

        let created_user: UserProfile = db
            .create("user")
            .content(test_user_content)
            .await
            .unwrap()
            .expect("Test user creation returned nothing (None).");

        let test_song_content = SongWithRelations {
            id: Some(create_song_thing("test")),
            title: "test".to_string(),
            file_url: "test".to_string(),
            duration: Duration::new(120, 0),
            song_index: 1,
            tempo: 0f32,
            total_listens: 0,
            total_user_listens: 0,
            total_likes: 0,
            artists: None,
            album: None, //TODO: ADD VERIFICATION IF ALBUM HAS +1 LISTENS
            added_at: None,
        };

        let created_song: SongWithRelations = db
            .create("song")
            .content(test_song_content)
            .await
            .unwrap()
            .expect("Test user creation returned nothing (None).");

        let user_id = created_user.id.unwrap().id.to_string();
        let song_id = created_song.id.unwrap().id.to_string();

        (db, user_id, song_id)
    }

    #[tokio::test]
    async fn test_listen_to_song() {
        let (db, user_id, song_id) = setup_db().await;

        // --- Test 1: No user: +1 listen to song
        let result = SongService::listen_to_song(&db, &song_id, None).await;
        assert!(result.is_ok(), "Test 1 failed: {:?}", result.err());

        let song_thing = create_song_thing(&song_id);
        let updated_song: Vec<SongWithRelations> = db
            .query("SELECT * FROM $song")
            .bind(("song", song_thing.clone()))
            .await
            .unwrap()
            .take(0)
            .unwrap();
        assert_eq!(
            updated_song[0].total_listens, 1,
            "Song should have 1 listen"
        );

        // --- Test 2: User first listen: added table and +1 listen to song
        let user_id = user_id;
        let result = SongService::listen_to_song(&db, &song_id, Some(&user_id)).await;
        assert!(result.is_ok(), "Test 2 failed: {:?}", result.err());

        let check_query = "SELECT count() as total FROM user_listens_song WHERE in = $user_id AND out = $song_id GROUP ALL";
        let mut check_response = db
            .query(check_query)
            .bind(("user_id", create_user_thing(&user_id)))
            .bind(("song_id", create_song_thing(&song_id)))
            .await
            .unwrap();
        let count_result: Option<CountResult> = check_response.take(0).unwrap();
        assert!(
            count_result.unwrap().total > 0,
            "User listen relation should exist"
        );

        let updated_song: Vec<SongWithRelations> = db
            .query("SELECT * FROM $song")
            .bind(("song", song_thing.clone()))
            .await
            .unwrap()
            .take(0)
            .unwrap();
        assert_eq!(
            updated_song[0].total_listens, 2,
            "Song should have 2 listens"
        );

        // --- Test 3: User x listen: added +1 to user table and +1 listen to song
        let result = SongService::listen_to_song(&db, &song_id, Some(&user_id)).await;
        assert!(result.is_ok(), "Test 3 failed: {:?}", result.err());

        let get_relation_query =
            "SELECT * FROM user_listens_song WHERE in = $user_id AND out = $song_id LIMIT 1";
        let mut relation_response = db
            .query(get_relation_query)
            .bind(("user_id", create_user_thing(&user_id)))
            .bind(("song_id", create_song_thing(&song_id)))
            .await
            .unwrap();

        #[derive(serde::Deserialize)]
        struct UserListenRelation {
            total_listens: u64,
            total_duration: Duration,
            recent_dates: Vec<Datetime>,
        }

        let relation: Option<UserListenRelation> = relation_response.take(0).unwrap();
        let relation = relation.unwrap();
        assert_eq!(
            relation.total_listens, 2,
            "User listen relation should have total_listens = 2"
        );
        assert_eq!(
            relation.total_duration,
            Duration::new(240, 0),
            "User listen relation should have total_duration = 240s (2 * 120s)"
        );
        assert_eq!(
            relation.recent_dates.len(),
            2,
            "User listen relation should have 2 recent dates"
        );

        let updated_song: Vec<SongWithRelations> = db
            .query("SELECT * FROM $song")
            .bind(("song", song_thing))
            .await
            .unwrap()
            .take(0)
            .unwrap();
        assert_eq!(
            updated_song[0].total_listens, 3,
            "Song should have 3 listens"
        );
    }
}
