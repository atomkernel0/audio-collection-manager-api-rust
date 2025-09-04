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

#[derive(Copy, Clone, Debug)]
enum FavoriteTable {
    Album,
    Song,
    Artist,
}


#[derive(Copy, Clone, Debug)]
enum SortField {
    FavoritedAt,
    SortOrder,
    LastAccessed,
    Title,
}

impl SortField {
    fn from_str(s: &str) -> Self {
        match s {
            "favoritedAt" => Self::FavoritedAt,
            "sortOrder" => Self::SortOrder,
            "lastAccessed" => Self::LastAccessed,
            "title" => Self::Title,
            _ => Self::FavoritedAt,
        }
    }

    fn order_by_for_albums(&self, dir: &str) -> &'static str {
        match (self, dir) {
            (Self::FavoritedAt, "ASC") => "ORDER BY favorited_at ASC",
            (Self::FavoritedAt, "DESC") => "ORDER BY favorited_at DESC",
            (Self::SortOrder, "ASC") => "ORDER BY sort_order ASC",
            (Self::SortOrder, "DESC") => "ORDER BY sort_order DESC",
            (Self::LastAccessed, "ASC") => "ORDER BY last_accessed ASC",
            (Self::LastAccessed, "DESC") => "ORDER BY last_accessed DESC",
            (Self::Title, "ASC") => "ORDER BY album.title ASC",
            (Self::Title, "DESC") => "ORDER BY album.title DESC",
            _ => "ORDER BY favorited_at DESC",
        }
    }

    fn order_by_for_songs(&self, dir: &str) -> &'static str {
        match (self, dir) {
            (Self::FavoritedAt, "ASC") => "ORDER BY favorited_at ASC",
            (Self::FavoritedAt, "DESC") => "ORDER BY favorited_at DESC",
            (Self::SortOrder, "ASC") => "ORDER BY sort_order ASC",
            (Self::SortOrder, "DESC") => "ORDER BY sort_order DESC",
            (Self::LastAccessed, "ASC") => "ORDER BY last_accessed ASC",
            (Self::LastAccessed, "DESC") => "ORDER BY last_accessed DESC",
            (Self::Title, "ASC") => "ORDER BY song.title ASC",
            (Self::Title, "DESC") => "ORDER BY song.title DESC",
            _ => "ORDER BY favorited_at DESC",
        }
    }

    fn order_by_for_artists(&self, dir: &str) -> &'static str {
        match (self, dir) {
            (Self::FavoritedAt, "ASC") => "ORDER BY favorited_at ASC",
            (Self::FavoritedAt, "DESC") => "ORDER BY favorited_at DESC",
            (Self::SortOrder, "ASC") => "ORDER BY sort_order ASC",
            (Self::SortOrder, "DESC") => "ORDER BY sort_order DESC",
            (Self::LastAccessed, "ASC") => "ORDER BY last_accessed ASC",
            (Self::LastAccessed, "DESC") => "ORDER BY last_accessed DESC",
            (Self::Title, "ASC") => "ORDER BY artist.name ASC",
            (Self::Title, "DESC") => "ORDER BY artist.name DESC",
            _ => "ORDER BY favorited_at DESC",
        }
    }
}

pub struct FavoriteService;

impl FavoriteService {



    fn validate_sort_direction(direction: &str) -> Result<&'static str, Error> {
        match direction.to_uppercase().as_str() {
            "ASC" => Ok("ASC"),
            "DESC" => Ok("DESC"),
            _ => Err(Error::InvalidInput {
                reason: format!(
                    "Invalid sort_direction: {}. Allowed values are ASC or DESC.",
                    direction
                ),
            }),
        }
    }

    async fn get_favorite_count_for_table(
        db: &Surreal<Any>,
        user_id: &str,
        table: FavoriteTable,
    ) -> Result<u64, Error> {
        let user_thing = create_user_thing(user_id);

        let count_sql = match table {
            FavoriteTable::Album => {
                "SELECT count() as total FROM user_likes_album WHERE `in` = $user_id GROUP ALL"
            }
            FavoriteTable::Song => {
                "SELECT count() as total FROM user_likes_song WHERE `in` = $user_id GROUP ALL"
            }
            FavoriteTable::Artist => {
                "SELECT count() as total FROM user_likes_artist WHERE `in` = $user_id GROUP ALL"
            }
        };

        let mut count_response = db.query(count_sql).bind(("user_id", user_thing)).await?;
        let count_result: Option<CountResult> = count_response.take(0)?;
        let total_items = count_result.map(|r| r.total).unwrap_or(0);

        Ok(total_items)
    }

    pub async fn get_favorite_albums_count(db: &Surreal<Any>, user_id: &str) -> Result<u64, Error> {
        Self::get_favorite_count_for_table(db, user_id, FavoriteTable::Album).await
    }

    pub async fn get_favorite_songs_count(db: &Surreal<Any>, user_id: &str) -> Result<u64, Error> {
        Self::get_favorite_count_for_table(db, user_id, FavoriteTable::Song).await
    }

    pub async fn get_favorite_artists_count(
        db: &Surreal<Any>,
        user_id: &str,
    ) -> Result<u64, Error> {
        Self::get_favorite_count_for_table(db, user_id, FavoriteTable::Artist).await
    }

    pub async fn get_favorite_albums(
        db: &Surreal<Any>,
        user_id: &str,
        query: &FavoritesQuery,
    ) -> Result<FavoritesResponse<AlbumWithFavoriteMetadata>, Error> {
        let user_thing = create_user_thing(user_id);
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).max(1).min(100);
        let sort_by_frontend = query.sort_by.as_deref().unwrap_or("favoritedAt");
        let sort_field = SortField::from_str(sort_by_frontend);
        let sort_direction = query.sort_direction.as_deref().unwrap_or("DESC");
        let sort_direction = Self::validate_sort_direction(sort_direction)?;
        let order_by = sort_field.order_by_for_albums(sort_direction);

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
                        total_duration: out.total_duration OR 0s,
                        total_listens: out.total_listens,
                        total_user_listens: out.total_user_listens,
                        total_likes: out.total_likes,
                        artists: (out<-artist_creates_album<-artist[*])
                    }} AS album,
                    IF sort_order != NONE THEN sort_order ELSE 0 END as sort_order,
                    last_accessed,
                    IF created_at != NONE THEN created_at ELSE time::now() END AS favorited_at
                FROM user_likes_album
                WHERE `in` = $user_id AND out.id IS NOT NONE
                {}
                LIMIT $limit START $offset
                "#,
                order_by
            );

            let mut data_response = db
                .query(&data_sql)
                .bind(("user_id", user_thing))
                .bind(("limit", page_size))
                .bind(("offset", offset))
                .await?;
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
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).max(1).min(100);
        let sort_by_frontend = query.sort_by.as_deref().unwrap_or("favoritedAt");
        let sort_field = SortField::from_str(sort_by_frontend);
        let sort_direction = query.sort_direction.as_deref().unwrap_or("DESC");
        let sort_direction = Self::validate_sort_direction(sort_direction)?;
        let order_by = sort_field.order_by_for_songs(sort_direction);

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
                        duration: out.duration OR 0s,
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
                WHERE `in` = $user_id AND out.id IS NOT NONE
                {}
                LIMIT $limit START $offset
                "#,
                order_by
            );

            println!("DEBUG: Fetching favorite songs for user_id: {}", user_id);

            let mut data_response = db
                .query(&data_sql)
                .bind(("user_id", user_thing))
                .bind(("limit", page_size))
                .bind(("offset", offset))
                .await
                .map_err(|e| {
                    eprintln!("DEBUG: Error fetching favorite songs: {}", e);
                    Error::DbError(format!(
                        "Erreur lors de la récupération des chansons favorites: {}",
                        e
                    ))
                })?;
            let songs_data: Vec<SongWithFavoriteMetadata> = data_response.take(0).map_err(|e| {
                eprintln!("DEBUG: Deserialization error in get_favorite_songs: {}", e);
                Error::DbError(format!(
                    "Erreur de désérialisation des chansons favorites: {}",
                    e
                ))
            })?;

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
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).max(1).min(100);
        let sort_by_frontend = query.sort_by.as_deref().unwrap_or("favoritedAt");
        let sort_field = SortField::from_str(sort_by_frontend);
        let sort_direction = query.sort_direction.as_deref().unwrap_or("DESC");
        let sort_direction = Self::validate_sort_direction(sort_direction)?;
        let order_by = sort_field.order_by_for_artists(sort_direction);

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
                    WHERE `in` = $user_id AND out.id IS NOT NONE
                    {}
                    LIMIT $limit START $offset
                    "#,
                order_by
            );

            let mut data_response = db
                .query(&data_sql)
                .bind(("user_id", user_thing))
                .bind(("limit", page_size))
                .bind(("offset", offset))
                .await?;
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
        table: FavoriteTable,
    ) -> Result<bool, Error> {
        let user_thing = create_user_thing(user_id);

        let (sql_check, sql_delete, sql_create) = match table {
            FavoriteTable::Album => (
                "SELECT count() as total FROM user_likes_album WHERE `in` = $user AND out = $item GROUP ALL",
                "DELETE user_likes_album WHERE `in` = $user AND out = $item RETURN NONE",
                "RELATE $user->user_likes_album->$item SET created_at = time::now()",
            ),
            FavoriteTable::Song => (
                "SELECT count() as total FROM user_likes_song WHERE `in` = $user AND out = $item GROUP ALL",
                "DELETE user_likes_song WHERE `in` = $user AND out = $item RETURN NONE",
                "RELATE $user->user_likes_song->$item SET created_at = time::now()",
            ),
            FavoriteTable::Artist => (
                "SELECT count() as total FROM user_likes_artist WHERE `in` = $user AND out = $item GROUP ALL",
                "DELETE user_likes_artist WHERE `in` = $user AND out = $item RETURN NONE",
                "RELATE $user->user_likes_artist->$item SET created_at = time::now()",
            ),
        };

        let mut response = db
            .query(sql_check)
            .bind(("user", user_thing.clone()))
            .bind(("item", item_thing.clone()))
            .await?;

        let result: Option<CountResult> = response.take(0)?;
        let exists = result.map_or(false, |r| r.total > 0);

        if exists {
            db.query(sql_delete)
                .bind(("user", user_thing))
                .bind(("item", item_thing))
                .await?;
            Ok(false)
        } else {
            db.query(sql_create)
                .bind(("user", user_thing))
                .bind(("item", item_thing))
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
        Self::toggle_favorite_item(db, user_id, album_thing, FavoriteTable::Album).await
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
        Self::toggle_favorite_item(db, user_id, song_thing, FavoriteTable::Song).await
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
        Self::toggle_favorite_item(db, user_id, artist_thing, FavoriteTable::Artist).await
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
            total_play_time: 0,         // TODO: Compute from durations
            most_played_genres: vec![], // TODO: Implement
            recently_added: RecentlyAddedFavorites {
                albums: vec![],
                songs: vec![],
                artists: vec![],
            },
        })
    }
}
