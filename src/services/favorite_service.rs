use crate::{
    helpers::{
        album_helpers::album_exists,
        artist_helpers::artist_exists,
        song_helpers::song_exists,
        thing_helpers::{
            create_album_thing, create_artist_thing, create_song_thing, create_user_thing,
            parse_id_part, thing_to_string,
        },
    },
    models::{favorite::*, pagination::PaginationInfo},
    Error,
};
use futures::try_join;
use serde::Deserialize;
use surrealdb::{engine::any::Any, sql::Thing, Surreal};

#[derive(Debug, Deserialize)]
struct CountResult {
    total: u64,
}

pub struct FavoriteService;

impl FavoriteService {
    fn map_sort_by_to_db_for_albums(sort_by: &str) -> &str {
        match sort_by {
            "favoritedAt" => "favorited_at",
            "sortOrder" => "sort_order",
            "lastAccessed" => "last_accessed",
            "title" => "album.title",
            _ => "favorited_at",
        }
    }

    fn map_sort_by_to_db_for_songs(sort_by: &str) -> &str {
        match sort_by {
            "favoritedAt" => "favorited_at",
            "sortOrder" => "sort_order",
            "lastAccessed" => "last_accessed",
            "title" => "song.title",
            _ => "favorited_at",
        }
    }

    fn map_sort_by_to_db_for_artists(sort_by: &str) -> &str {
        match sort_by {
            "favoritedAt" => "favorited_at",
            "sortOrder" => "sort_order",
            "lastAccessed" => "last_accessed",
            // Name resides under the selected 'artist' object
            "title" => "artist.name",
            _ => "favorited_at",
        }
    }

    async fn get_favorite_count_for_table(
        db: &Surreal<Any>,
        user_id: &str,
        table_name: &str,
    ) -> Result<u64, Error> {
        let user_thing = create_user_thing(user_id);

        let count_sql = format!(
            "SELECT count() as total FROM {} WHERE `in` = $user_id GROUP ALL",
            table_name
        );
        let mut count_response = db
            .query(&count_sql)
            .bind(("user_id", user_thing.clone()))
            .await?;
        let count_result: Option<CountResult> = count_response.take(0)?;
        let total_items = count_result.map(|r| r.total).unwrap_or(0);

        Ok(total_items)
    }

    pub async fn get_favorite_albums_count(db: &Surreal<Any>, user_id: &str) -> Result<u64, Error> {
        Self::get_favorite_count_for_table(db, user_id, "user_likes_album").await
    }

    pub async fn get_favorite_songs_count(db: &Surreal<Any>, user_id: &str) -> Result<u64, Error> {
        Self::get_favorite_count_for_table(db, user_id, "user_likes_song").await
    }

    pub async fn get_favorite_artists_count(
        db: &Surreal<Any>,
        user_id: &str,
    ) -> Result<u64, Error> {
        Self::get_favorite_count_for_table(db, user_id, "user_likes_artist").await
    }

    pub async fn get_favorite_albums(
        db: &Surreal<Any>,
        user_id: &str,
        query: &FavoritesQuery,
    ) -> Result<FavoritesResponse<AlbumWithFavoriteMetadata>, Error> {
        let user_thing = create_user_thing(user_id);
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let sort_by_frontend = query.sort_by.as_deref().unwrap_or("favoritedAt");
        let sort_by = Self::map_sort_by_to_db_for_albums(sort_by_frontend);
        let sort_direction = query
            .sort_direction
            .as_deref()
            .unwrap_or("desc")
            .to_uppercase();

        let offset = (page - 1) * page_size;

        let total_items = Self::get_favorite_albums_count(db, user_id).await?;

        let albums: Vec<AlbumWithFavoriteMetadata> = if total_items == 0 {
            Vec::new()
        } else {
            let data_sql = format!(
                r#"
                SELECT
                    {{
                        id: out.id,
                        title: out.title,
                        cover_url: out.cover_url,
                        release_year: out.release_year,
                        genres: out.genres,
                        langs: out.langs,
                        dominant_color: out.dominant_color,
                        total_tracks: out.total_tracks,
                        total_duration: out.total_duration,
                        total_listens: out.total_listens,
                        total_user_listens: out.total_user_listens,
                        total_likes: out.total_likes,
                        artists: (out<-artist_creates_album<-artist[*])
                    }} AS album,
                    IF sort_order != NONE THEN sort_order ELSE 0 END as sort_order,
                    last_accessed,
                    IF created_at != NONE THEN created_at ELSE time::now() END AS favorited_at
                FROM user_likes_album
                WHERE `in` = $user_id
                ORDER BY {} {}
                LIMIT {} START {}
                "#,
                sort_by, sort_direction, page_size, offset
            );

            let mut data_response = db.query(&data_sql).bind(("user_id", user_thing)).await?;
            let albums_data: Vec<AlbumWithFavoriteMetadata> = data_response.take(0)?;

            albums_data
        };

        let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as u32;

        Ok(FavoritesResponse {
            data: albums,
            pagination: PaginationInfo {
                current_page: page,
                total_pages,
                total_items,
                page_size,
                has_next_page: page < total_pages,
                has_previous_page: page > 1,
            },
        })
    }

    pub async fn get_favorite_album_ids(
        db: &Surreal<Any>,
        user_id: &str,
    ) -> Result<Vec<String>, Error> {
        let user_thing = create_user_thing(user_id);

        let total_items = Self::get_favorite_albums_count(db, user_id).await?;

        if total_items == 0 {
            Ok(Vec::new())
        } else {
            let data_sql = "SELECT VALUE out.id FROM user_likes_album WHERE `in` = $user_id";

            let mut data_response = db.query(data_sql).bind(("user_id", user_thing)).await?;
            let album_things: Vec<Thing> = data_response.take(0)?;

            let album_ids: Vec<String> = album_things
                .iter()
                .map(|thing| {
                    let thing_str = thing_to_string(thing);
                    let id_part = parse_id_part(&thing_str);
                    id_part.to_string()
                })
                .collect();

            Ok(album_ids)
        }
    }

    pub async fn get_favorite_songs(
        db: &Surreal<Any>,
        user_id: &str,
        query: &FavoritesQuery,
    ) -> Result<FavoritesResponse<SongWithFavoriteMetadata>, Error> {
        let user_thing = create_user_thing(user_id);
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let sort_by_frontend = query.sort_by.as_deref().unwrap_or("favoritedAt");
        let sort_by = Self::map_sort_by_to_db_for_songs(sort_by_frontend);
        let sort_direction = query
            .sort_direction
            .as_deref()
            .unwrap_or("desc")
            .to_uppercase();

        let offset = (page - 1) * page_size;

        let total_items = Self::get_favorite_songs_count(db, user_id).await?;

        let songs = if total_items == 0 {
            Vec::new()
        } else {
            let data_sql = format!(
                r#"
                SELECT
                    {{
                        id: out.id,
                        title: out.title,
                        file_url: out.file_url,
                        duration: out.duration,
                        song_index: out.song_index,
                        tempo: out.tempo,
                        total_listens: out.total_listens,
                        total_user_listens: out.total_user_listens,
                        total_likes: out.total_likes,
                        artists: (out<-artist_performs_song<-artist[*]),
                        album: (SELECT * FROM (out<-album_contains_song<-album)[0])[0]
                    }} AS song,
                    IF sort_order != NONE THEN sort_order ELSE 0 END as sort_order,
                    last_accessed,
                    IF created_at != NONE THEN created_at ELSE time::now() END AS favorited_at
                FROM user_likes_song
                WHERE `in` = $user_id
                ORDER BY {} {}
                LIMIT {} START {}
                "#,
                sort_by, sort_direction, page_size, offset
            );

            let mut data_response = db.query(&data_sql).bind(("user_id", user_thing)).await?;
            let songs_data: Vec<SongWithFavoriteMetadata> = data_response.take(0)?;

            songs_data
        };

        let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as u32;

        Ok(FavoritesResponse {
            data: songs,
            pagination: PaginationInfo {
                current_page: page,
                total_pages,
                total_items,
                page_size,
                has_next_page: page < total_pages,
                has_previous_page: page > 1,
            },
        })
    }

    pub async fn get_favorite_song_ids(
        db: &Surreal<Any>,
        user_id: &str,
    ) -> Result<Vec<String>, Error> {
        let user_thing = create_user_thing(user_id);

        let total_items = Self::get_favorite_songs_count(db, user_id).await?;

        if total_items == 0 {
            Ok(Vec::new())
        } else {
            let data_sql = "SELECT VALUE out.id FROM user_likes_song WHERE `in` = $user_id";

            let mut data_response = db.query(data_sql).bind(("user_id", user_thing)).await?;
            let song_things: Vec<Thing> = data_response.take(0)?;

            let song_ids: Vec<String> = song_things
                .iter()
                .map(|thing| {
                    let thing_str = thing_to_string(thing);
                    let id_part = parse_id_part(&thing_str);
                    id_part.to_string()
                })
                .collect();

            Ok(song_ids)
        }
    }

    pub async fn get_favorite_artists(
        db: &Surreal<Any>,
        user_id: &str,
        query: &FavoritesQuery,
    ) -> Result<FavoritesResponse<ArtistWithFavoriteMetadata>, Error> {
        let user_thing = create_user_thing(user_id);
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let sort_by_frontend = query.sort_by.as_deref().unwrap_or("favoritedAt");
        let sort_by = Self::map_sort_by_to_db_for_artists(sort_by_frontend);
        let sort_direction = query
            .sort_direction
            .as_deref()
            .unwrap_or("desc")
            .to_uppercase();

        let offset = (page - 1) * page_size;

        let total_items = Self::get_favorite_artists_count(db, user_id).await?;

        let artists = if total_items == 0 {
            Vec::new()
        } else {
            let data_sql = format!(
                r#"
                SELECT
                    {{
                        id: out.id,
                        name: out.name,
                        genres: out.genres,
                        country_code: out.country_code,
                        artist_image: out.artist_image,
                        albums_count: out.albums_count,
                        songs_count: out.songs_count,
                        total_likes: IF out.total_likes != NONE THEN out.total_likes ELSE 0 END,
                        albums: (out->artist_creates_album->album.*)
                    }} AS artist,
                    IF sort_order != NONE THEN sort_order ELSE 0 END as sort_order,
                    last_accessed,
                    IF created_at != NONE THEN created_at ELSE time::now() END as favorited_at
                FROM user_likes_artist
                WHERE `in` = $user_id
                ORDER BY {} {}
                LIMIT {} START {}
                "#,
                sort_by, sort_direction, page_size, offset
            );

            let mut data_response = db.query(&data_sql).bind(("user_id", user_thing)).await?;
            let artists_data: Vec<ArtistWithFavoriteMetadata> = data_response.take(0)?;

            artists_data
        };

        let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as u32;

        Ok(FavoritesResponse {
            data: artists,
            pagination: PaginationInfo {
                current_page: page,
                total_pages,
                total_items,
                page_size,
                has_next_page: page < total_pages,
                has_previous_page: page > 1,
            },
        })
    }

    pub async fn get_favorite_artist_ids(
        db: &Surreal<Any>,
        user_id: &str,
    ) -> Result<Vec<String>, Error> {
        let user_thing = create_user_thing(user_id);

        let total_items = Self::get_favorite_artists_count(db, user_id).await?;

        if total_items == 0 {
            Ok(Vec::new())
        } else {
            let data_sql = "SELECT VALUE out.id FROM user_likes_artist WHERE `in` = $user_id";

            let mut data_response = db.query(data_sql).bind(("user_id", user_thing)).await?;
            let artist_things: Vec<Thing> = data_response.take(0)?;

            let artist_ids: Vec<String> = artist_things
                .iter()
                .map(|thing| {
                    let thing_str = thing_to_string(thing);
                    let id_part = parse_id_part(&thing_str);
                    id_part.to_string()
                })
                .collect();

            Ok(artist_ids)
        }
    }

    async fn toggle_favorite_item(
        db: &Surreal<Any>,
        user_id: &str,
        item_thing: Thing,
        table_name: &str,
        item_bind_name: &str,
    ) -> Result<bool, Error> {
        let user_thing = create_user_thing(user_id);
        let item_bind_name = item_bind_name.to_string();

        let sql_check = format!(
            "SELECT count() as total FROM {} WHERE `in` = $user AND out = ${} GROUP ALL",
            table_name, &item_bind_name
        );

        let mut response = db
            .query(&sql_check)
            .bind(("user", user_thing.clone()))
            .bind((item_bind_name.clone(), item_thing.clone()))
            .await?;

        let result: Option<CountResult> = response.take(0)?;
        let exists = result.map_or(false, |r| r.total > 0);

        if exists {
            let sql_delete = format!(
                "DELETE {} WHERE `in` = $user AND out = ${} RETURN NONE",
                table_name, &item_bind_name
            );
            db.query(&sql_delete)
                .bind(("user", user_thing))
                .bind((item_bind_name, item_thing))
                .await?;
            Ok(false)
        } else {
            let sql_create = format!(
                "RELATE $user->{}->${} SET created_at = time::now()",
                table_name, &item_bind_name
            );
            db.query(&sql_create)
                .bind(("user", user_thing))
                .bind((item_bind_name, item_thing))
                .await?
                .check()?;
            Ok(true)
        }
    }

    pub async fn toggle_favorite_album(
        db: &Surreal<Any>,
        user_id: &str,
        album_id: &str,
    ) -> Result<bool, Error> {
        if !album_exists(db, album_id).await? {
            return Err(Error::AlbumNotFound {
                id: album_id.to_string(),
            });
        }

        let album_thing = create_album_thing(album_id);
        Self::toggle_favorite_item(db, user_id, album_thing, "user_likes_album", "album").await
    }

    pub async fn toggle_favorite_song(
        db: &Surreal<Any>,
        user_id: &str,
        song_id: &str,
    ) -> Result<bool, Error> {
        if !song_exists(db, song_id).await? {
            return Err(Error::SongNotFound {
                id: song_id.to_string(),
            });
        }

        let song_thing = create_song_thing(song_id);
        Self::toggle_favorite_item(db, user_id, song_thing, "user_likes_song", "song").await
    }

    pub async fn toggle_favorite_artist(
        db: &Surreal<Any>,
        user_id: &str,
        artist_id: &str,
    ) -> Result<bool, Error> {
        if !artist_exists(db, artist_id).await? {
            return Err(Error::ArtistNotFound {
                id: artist_id.to_string(),
            });
        }

        let artist_thing = create_artist_thing(artist_id);
        Self::toggle_favorite_item(db, user_id, artist_thing, "user_likes_artist", "artist").await
    }

    pub async fn check_favorite_album(
        db: &Surreal<Any>,
        user_id: &str,
        album_id: &str,
    ) -> Result<bool, Error> {
        if !album_exists(db, album_id).await? {
            return Err(Error::AlbumNotFound {
                id: album_id.to_string(),
            });
        }

        let user_thing = create_user_thing(user_id);
        let album_thing = create_album_thing(album_id);

        let sql_check = "SELECT count() as total FROM user_likes_album WHERE `in` = $user AND out = $album GROUP ALL";

        let mut response = db
            .query(sql_check)
            .bind(("user", user_thing))
            .bind(("album", album_thing))
            .await?;

        let result: Option<CountResult> = response.take(0)?;
        let exists = result.map_or(false, |r| r.total > 0);

        Ok(exists)
    }

    pub async fn check_favorite_artist(
        db: &Surreal<Any>,
        user_id: &str,
        artist_id: &str,
    ) -> Result<bool, Error> {
        if !artist_exists(db, artist_id).await? {
            return Err(Error::ArtistNotFound {
                id: artist_id.to_string(),
            });
        }

        let user_thing = create_user_thing(user_id);
        let artist_thing = create_artist_thing(artist_id);

        let sql_check = "SELECT count() as total FROM user_likes_artist WHERE `in` = $user AND out = $artist GROUP ALL";

        let mut response = db
            .query(sql_check)
            .bind(("user", user_thing))
            .bind(("artist", artist_thing))
            .await?;

        let result: Option<CountResult> = response.take(0)?;
        let exists = result.map_or(false, |r| r.total > 0);

        Ok(exists)
    }

    pub async fn check_favorite_song(
        db: &Surreal<Any>,
        user_id: &str,
        song_id: &str,
    ) -> Result<bool, Error> {
        if !song_exists(db, song_id).await? {
            return Err(Error::SongNotFound {
                id: song_id.to_string(),
            });
        }

        let user_thing = create_user_thing(user_id);
        let song_thing = create_song_thing(song_id);

        let sql_check = "SELECT count() as total FROM user_likes_song WHERE `in` = $user AND out = $song GROUP ALL";

        let mut response = db
            .query(sql_check)
            .bind(("user", user_thing))
            .bind(("song", song_thing))
            .await?;

        let result: Option<CountResult> = response.take(0)?;
        let exists = result.map_or(false, |r| r.total > 0);

        Ok(exists)
    }

    pub async fn get_statistics(
        db: &Surreal<Any>,
        user_id: &str,
    ) -> Result<FavoritesStatistics, Error> {
        let albums_count_future = Self::get_favorite_albums_count(db, user_id);
        let songs_count_future = Self::get_favorite_songs_count(db, user_id);
        let artists_count_future = Self::get_favorite_artists_count(db, user_id);

        let (albums_count, songs_count, artists_count) = try_join!(
            albums_count_future,
            songs_count_future,
            artists_count_future
        )?;

        Ok(FavoritesStatistics {
            total_albums: albums_count,
            total_songs: songs_count,
            total_artists: artists_count,
            total_play_time: 0,         //TODO: Calculer à partir des durées
            most_played_genres: vec![], //TODO: Implémenter
            recently_added: RecentlyAddedFavorites {
                albums: vec![],
                songs: vec![],
                artists: vec![],
            },
        })
    }
}
