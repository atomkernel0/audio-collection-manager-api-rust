use crate::{
    helpers::{
        album_helpers::album_exists,
        artist_helpers::artist_exists,
        song_helpers::song_exists,
        thing_helpers::{
            create_album_thing, create_artist_thing, create_song_thing, create_user_thing,
        },
    },
    models::{album::Album, artist::Artist, favorite::*, pagination::PaginationInfo, song::Song},
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
    fn map_sort_by_to_db(sort_by: &str) -> &str {
        match sort_by {
            "favoritedAt" => "created_at",
            "sortOrder" => "sort_order",
            "lastAccessed" => "last_accessed",
            "title" => "title",
            _ => "created_at",
        }
    }

    async fn get_favorite_count_for_table(
        db: &Surreal<Any>,
        user_id: &str,
        table_name: &str,
    ) -> Result<u64, Error> {
        let user_thing = create_user_thing(user_id);

        let count_sql = format!(
            "SELECT count() as total FROM {} WHERE in = $user_id GROUP ALL",
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

    pub async fn get_favorite_albums(
        db: &Surreal<Any>,
        user_id: &str,
        query: &FavoritesQuery,
    ) -> Result<FavoritesResponse<AlbumWithFavoriteMetadata>, Error> {
        let user_thing = create_user_thing(user_id);
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let sort_by_frontend = query.sort_by.as_deref().unwrap_or("favoritedAt");
        let sort_by = Self::map_sort_by_to_db(sort_by_frontend);
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
                    out.* as album,
                    sort_order,
                    last_accessed,
                    created_at as favorited_at,
                    (SELECT * FROM artist WHERE id IN (
                        SELECT out FROM artist_creates_album WHERE in = out.id
                    )) as artists
                FROM user_likes_album
                WHERE in = $user_id
                ORDER BY {} {}
                LIMIT {} START {}
                "#,
                sort_by, sort_direction, page_size, offset
            );

            let mut data_response = db.query(&data_sql).bind(("user_id", user_thing)).await?;
            let albums_data: Vec<AlbumWithFavoriteMetadata> = data_response.take(0)?;

            albums_data
                .into_iter()
                .map(|data| AlbumWithFavoriteMetadata {
                    album: Album {
                        id: data.album.id,
                        title: data.album.title,
                        cover_url: data.album.cover_url,
                        release_year: data.album.release_year,
                        genres: data.album.genres,
                        langs: data.album.langs,
                        dominant_color: data.album.dominant_color,
                        total_tracks: data.album.total_tracks,
                        total_duration: data.album.total_duration,
                        total_listens: data.album.total_listens,
                        total_user_listens: data.album.total_user_listens,
                        total_likes: data.album.total_likes,
                    },
                    artists: data.artists,
                    sort_order: data.sort_order,
                    last_accessed: data.last_accessed,
                    favorited_at: data.favorited_at,
                })
                .collect()
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

    pub async fn get_favorite_songs(
        db: &Surreal<Any>,
        user_id: &str,
        query: &FavoritesQuery,
    ) -> Result<FavoritesResponse<SongWithFavoriteMetadata>, Error> {
        let user_thing = create_user_thing(user_id);
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let sort_by_frontend = query.sort_by.as_deref().unwrap_or("favoritedAt");
        let sort_by = Self::map_sort_by_to_db(sort_by_frontend);
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
                    out.* as song,
                    sort_order,
                    last_accessed,
                    created_at as favorited_at
                FROM user_likes_song
                WHERE in = $user_id
                ORDER BY {} {}
                LIMIT {} START {}
                "#,
                sort_by, sort_direction, page_size, offset
            );

            let mut data_response = db.query(&data_sql).bind(("user_id", user_thing)).await?;
            let songs_data: Vec<SongWithFavoriteMetadata> = data_response.take(0)?;

            songs_data
                .into_iter()
                .map(|data| SongWithFavoriteMetadata {
                    song: Song {
                        id: data.song.id,
                        title: data.song.title,
                        file_url: data.song.file_url,
                        duration: data.song.duration,
                        song_index: data.song.song_index,
                        tempo: data.song.tempo,
                        total_listens: data.song.total_listens,
                        total_user_listens: data.song.total_user_listens,
                        total_likes: data.song.total_likes,
                    },
                    sort_order: data.sort_order,
                    last_accessed: data.last_accessed,
                    favorited_at: data.favorited_at,
                })
                .collect()
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

    pub async fn get_favorite_artists(
        db: &Surreal<Any>,
        user_id: &str,
        query: &FavoritesQuery,
    ) -> Result<FavoritesResponse<ArtistWithFavoriteMetadata>, Error> {
        let user_thing = create_user_thing(user_id);
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let sort_by_frontend = query.sort_by.as_deref().unwrap_or("favoritedAt");
        let sort_by = Self::map_sort_by_to_db(sort_by_frontend);
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
                    out.* as artist,
                    sort_order,
                    last_accessed,
                    created_at as favorited_at
                FROM user_likes_artist
                WHERE in = $user_id
                ORDER BY {} {}
                LIMIT {} START {}
                "#,
                sort_by, sort_direction, page_size, offset
            );

            let mut data_response = db.query(&data_sql).bind(("user_id", user_thing)).await?;
            let artists_data: Vec<ArtistWithFavoriteMetadata> = data_response.take(0)?;

            artists_data
                .into_iter()
                .map(|data| ArtistWithFavoriteMetadata {
                    artist: Artist {
                        id: data.artist.id,
                        name: data.artist.name,
                        genres: data.artist.genres,
                        country_code: data.artist.country_code,
                        artist_image: data.artist.artist_image,
                        albums_count: data.artist.albums_count,
                        songs_count: data.artist.songs_count,
                        total_likes: data.artist.total_likes,
                    },
                    sort_order: data.sort_order,
                    last_accessed: data.last_accessed,
                    favorited_at: data.favorited_at,
                })
                .collect()
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
            "SELECT count() as total FROM {} WHERE in = $user AND out = ${} GROUP ALL",
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
                "DELETE {} WHERE in = $user AND out = ${} RETURN NONE",
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

        let sql_check = "SELECT count() as total FROM user_likes_album WHERE in = $user AND out = $album GROUP ALL";

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

        let sql_check = "SELECT count() as total FROM user_likes_artist WHERE in = $user AND out = $artist GROUP ALL";

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

        let sql_check = "SELECT count() as total FROM user_likes_song WHERE in = $user AND out = $song GROUP ALL";

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
