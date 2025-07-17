use crate::{
    helpers::{
        album_helpers::album_exists,
        artist_helpers::artist_exists,
        favorite_helpers::{is_album_favorited, is_artist_favorited, is_song_favorited},
        song_helpers::song_exists,
        thing_helpers::{
            create_album_thing, create_artist_thing, create_song_thing, create_user_thing,
        },
    },
    models::{album::AlbumWithArtists, artist::Artist, favorite::*, song::Song},
    Error,
};
use serde::Deserialize;
use surrealdb::{engine::any::Any, sql::Thing, Datetime, Surreal};

#[derive(Debug, Deserialize)]
struct CountResult {
    total: u64,
}

/// Service pour la gestion des favoris
pub struct FavoriteService;

impl FavoriteService {
    /// Mappe les valeurs de tri du frontend vers les colonnes de la base de données
    fn map_sort_by_to_db(sort_by: &str) -> &str {
        match sort_by {
            "favoritedAt" => "created_at",
            "userRating" => "user_rating",
            "sortOrder" => "sort_order",
            "lastAccessed" => "last_accessed",
            "title" => "title",
            _ => "created_at", // valeur par défaut
        }
    }

    /// Récupère les albums favoris avec pagination et tri
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

        // Requête pour compter le total
        let count_sql =
            "SELECT count() as total FROM user_likes_album WHERE in = $user_id GROUP ALL";
        let mut count_response = db
            .query(count_sql)
            .bind(("user_id", user_thing.clone()))
            .await?;
        let count_result: Option<CountResult> = count_response.take(0)?;
        let total_items = count_result.map(|r| r.total).unwrap_or(0);

        println!("{:?}", total_items);

        // Si aucun favori, retourner directement une liste vide
        let albums: Vec<AlbumWithFavoriteMetadata> = if total_items == 0 {
            Vec::new()
        } else {
            // Requête pour récupérer les données
            let data_sql = format!(
                r#"
                SELECT
                    out.* as album,
                    tags,
                    notes,
                    user_rating,
                    sort_order,
                    is_favorite,
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

            println!("{:?}", data_sql);

            let mut data_response = db.query(&data_sql).bind(("user_id", user_thing)).await?;
            let albums_data: Vec<AlbumWithFavoriteData> = data_response.take(0)?;

            // Transformation des données
            albums_data
                .into_iter()
                .map(|data| AlbumWithFavoriteMetadata {
                    album: AlbumWithArtists {
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
                        artists: data.artists,
                    },
                    favorite_metadata: Some(FavoriteMetadata {
                        tags: data.tags,
                        notes: data.notes,
                        user_rating: data.user_rating,
                        sort_order: data.sort_order,
                        is_favorite: data.is_favorite,
                        last_accessed: data.last_accessed,
                        created_at: data.favorited_at,
                    }),
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
        let user_thing = create_user_thing(user_id);

        let count_sql =
            "SELECT count() as total FROM user_likes_album WHERE in = $user_id GROUP ALL";
        let mut count_response = db
            .query(count_sql)
            .bind(("user_id", user_thing.clone()))
            .await?;
        let count_result: Option<CountResult> = count_response.take(0)?;
        let total_items = count_result.map(|r| r.total).unwrap_or(0);

        Ok(total_items)
    }

    pub async fn get_is_favorite_album(
        db: &Surreal<Any>,
        album_id: &str,
        user_id: &str,
    ) -> Result<bool, Error> {
        let album_thing = create_album_thing(album_id);
        let user_thing = create_user_thing(user_id);

        let sql_query =
            "SELECT is_favorite FROM user_likes_album WHERE in = $user_id AND out = $album_id;";

        let mut result = db
            .query(sql_query)
            .bind(("user_id", user_thing))
            .bind(("album_id", album_thing))
            .await?;

        let favorite: Option<bool> = result.take("is_favorite")?;

        Ok(favorite.unwrap_or(false))
    }

    /// Récupère les chansons favorites avec pagination et tri
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

        // Requête pour compter le total
        let count_sql =
            "SELECT count() as total FROM user_likes_song WHERE in = $user_id GROUP ALL";
        let mut count_response = db
            .query(count_sql)
            .bind(("user_id", user_thing.clone()))
            .await?;
        let count_result: Option<CountResult> = count_response.take(0)?;
        let total_items = count_result.map(|r| r.total).unwrap_or(0);

        // Si aucun favori, retourner directement une liste vide
        let songs = if total_items == 0 {
            Vec::new()
        } else {
            // Requête pour récupérer les données
            let data_sql = format!(
                r#"
                SELECT
                    out.* as song,
                    tags,
                    notes,
                    user_rating,
                    sort_order,
                    is_favorite,
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
            let songs_data: Vec<SongWithFavoriteData> = data_response.take(0)?;

            // Transformation des données
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
                    favorite_metadata: Some(FavoriteMetadata {
                        tags: data.tags,
                        notes: data.notes,
                        user_rating: data.user_rating,
                        sort_order: data.sort_order,
                        is_favorite: data.is_favorite,
                        last_accessed: data.last_accessed,
                        created_at: data.favorited_at,
                    }),
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

    /// Récupère les artistes favoris avec pagination et tri
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

        // Requête pour compter le total
        let count_sql =
            "SELECT count() as total FROM user_likes_artist WHERE in = $user_id GROUP ALL";
        let mut count_response = db
            .query(count_sql)
            .bind(("user_id", user_thing.clone()))
            .await?;
        let count_result: Option<CountResult> = count_response.take(0)?;
        let total_items = count_result.map(|r| r.total).unwrap_or(0);

        // Si aucun favori, retourner directement une liste vide
        let artists = if total_items == 0 {
            Vec::new()
        } else {
            // Requête pour récupérer les données
            let data_sql = format!(
                r#"
                SELECT
                    out.* as artist,
                    tags,
                    notes,
                    user_rating,
                    sort_order,
                    is_favorite,
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
            let artists_data: Vec<ArtistWithFavoriteData> = data_response.take(0)?;

            // Transformation des données
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
                    },
                    favorite_metadata: Some(FavoriteMetadata {
                        tags: data.tags,
                        notes: data.notes,
                        user_rating: data.user_rating,
                        sort_order: data.sort_order,
                        is_favorite: data.is_favorite,
                        last_accessed: data.last_accessed,
                        created_at: data.favorited_at,
                    }),
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

    /// Ajoute un album aux favoris
    pub async fn add_favorite_album(
        db: &Surreal<Any>,
        user_id: &str,
        request: AddFavoriteRequest,
    ) -> Result<AlbumWithFavoriteMetadata, Error> {
        let user_thing = create_user_thing(user_id);
        let album_thing = create_album_thing(&request.item_id.id.to_string());

        // Vérifier si l'album existe
        if !album_exists(db, &request.item_id.id.to_string()).await? {
            return Err(Error::AlbumNotFound {
                id: request.item_id.id.to_string().clone(),
            });
        }

        // Vérifier si l'album est déjà dans les favoris
        if is_album_favorited(db, user_id, &request.item_id.id.to_string()).await? {
            return Err(Error::FavoriteAlreadyExists {
                item_type: "album".to_string(),
                item_id: request.item_id.id.to_string(),
            });
        }

        // Créer la relation avec métadonnées
        let tags = request.tags.unwrap_or_default();
        let create_sql = r#"
            RELATE $user_id->user_likes_album->$album_id SET
                tags = $tags,
                notes = $notes,
                user_rating = $user_rating,
                sort_order = $sort_order,
                is_favorite = $is_favorite,
                created_at = time::now()
        "#;

        db.query(create_sql)
            .bind(("user_id", user_thing))
            .bind(("album_id", album_thing.clone()))
            .bind(("tags", tags))
            .bind(("notes", request.notes))
            .bind(("user_rating", request.user_rating))
            .bind(("sort_order", request.sort_order.unwrap_or(0)))
            .bind(("is_favorite", request.is_favorite.unwrap_or(false)))
            .await?;

        // Récupérer l'album avec ses métadonnées
        Self::get_album_with_metadata(db, user_id, &album_thing).await
    }

    /// Supprime un album des favoris
    pub async fn remove_favorite_album(
        db: &Surreal<Any>,
        user_id: &str,
        album_id: &str,
    ) -> Result<(), Error> {
        let user_thing = create_user_thing(user_id);
        let album_thing = create_album_thing(album_id);

        // Vérifier si l'album est dans les favoris
        if !is_album_favorited(db, user_id, album_id).await? {
            return Err(Error::FavoriteNotFound {
                item_type: "album".to_string(),
                item_id: album_id.to_string(),
            });
        }

        let delete_sql = "DELETE user_likes_album WHERE in = $user_id AND out = $album_id";
        db.query(delete_sql)
            .bind(("user_id", user_thing))
            .bind(("album_id", album_thing))
            .await?;

        Ok(())
    }

    /// Toggle favorite album (if not liked => likes it)
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

        let user_id_part = user_id.split(':').nth(1).unwrap_or(user_id);
        let user_thing = Thing::from(("user", user_id_part));
        let album_thing = Thing::from(("album", album_id));

        let sql_check =
        "SELECT count() as total FROM user_likes_album WHERE in = $user AND out = $album GROUP ALL";

        let mut response = db
            .query(sql_check)
            .bind(("user", user_thing.clone()))
            .bind(("album", album_thing.clone()))
            .await?;

        let result: Option<CountResult> = response.take(0)?;
        let exists = result.map(|r| r.total > 0).unwrap_or(false);

        if exists {
            let sql_delete =
                "DELETE user_likes_album WHERE in = $user AND out = $album RETURN NONE";
            db.query(sql_delete)
                .bind(("user", user_thing))
                .bind(("album", album_thing))
                .await?;

            Ok(false)
        } else {
            let sql_create = "RELATE $user->user_likes_album->$album SET created_at = time::now()";
            db.query(sql_create)
                .bind(("user", user_thing))
                .bind(("album", album_thing))
                .await?
                .check()?;
            Ok(true)
        }
    }

    pub async fn like_song(db: &Surreal<Any>, user_id: &str, song_id: &str) -> Result<bool, Error> {
        let user_id_part = user_id.split(':').nth(1).unwrap_or(user_id);
        let user_thing = Thing::from(("user", user_id_part));
        let song_thing = Thing::from(("song", song_id));

        let sql_check =
        "SELECT count() as total FROM user_likes_song WHERE in = $user AND out = $song GROUP ALL";

        let mut response = db
            .query(sql_check)
            .bind(("user", user_thing.clone()))
            .bind(("song", song_thing.clone()))
            .await?;

        let result: Option<CountResult> = response.take(0)?;
        let exists = result.map(|r| r.total > 0).unwrap_or(false);

        if exists {
            let sql_delete = "DELETE user_likes_song WHERE in = $user AND out = $song RETURN NONE";
            db.query(sql_delete)
                .bind(("user", user_thing))
                .bind(("song", song_thing))
                .await?;

            Ok(false)
        } else {
            let sql_create = "RELATE $user->user_likes_song->$song SET created_at = time::now()";
            db.query(sql_create)
                .bind(("user", user_thing))
                .bind(("song", song_thing))
                .await?
                .check()?;
            Ok(true)
        }
    }

    /// Met à jour les métadonnées d'un album favori
    pub async fn update_favorite_album(
        db: &Surreal<Any>,
        user_id: &str,
        album_id: &str,
        request: UpdateFavoriteRequest,
    ) -> Result<AlbumWithFavoriteMetadata, Error> {
        let user_thing = create_user_thing(user_id);
        let album_thing = create_album_thing(album_id);

        // Vérifier si l'album est dans les favoris
        if !is_album_favorited(db, user_id, album_id).await? {
            return Err(Error::FavoriteNotFound {
                item_type: "album".to_string(),
                item_id: album_id.to_string(),
            });
        }

        // Mise à jour des métadonnées
        let update_sql = r#"
            UPDATE user_likes_album SET
                tags = COALESCE($tags, tags),
                notes = COALESCE($notes, notes),
                user_rating = COALESCE($user_rating, user_rating),
                sort_order = COALESCE($sort_order, sort_order),
                is_favorite = COALESCE($is_favorite, is_favorite)
            WHERE in = $user_id AND out = $album_id
        "#;

        db.query(update_sql)
            .bind(("user_id", user_thing))
            .bind(("album_id", album_thing.clone()))
            .bind(("tags", request.tags))
            .bind(("notes", request.notes))
            .bind(("user_rating", request.user_rating))
            .bind(("sort_order", request.sort_order))
            .bind(("is_favorite", request.is_favorite))
            .await?;

        // Récupérer l'album mis à jour
        Self::get_album_with_metadata(db, user_id, &album_thing).await
    }

    /// Ajoute une chanson aux favoris
    pub async fn add_favorite_song(
        db: &Surreal<Any>,
        user_id: &str,
        request: AddFavoriteRequest,
    ) -> Result<SongWithFavoriteMetadata, Error> {
        let user_thing = create_user_thing(user_id);
        let song_thing = create_song_thing(&request.item_id.id.to_string());

        // Vérifier si la chanson existe
        if !song_exists(db, &request.item_id.id.to_string()).await? {
            return Err(Error::SongNotFound {
                id: request.item_id.id.to_string().clone(),
            });
        }

        // Vérifier si la chanson est déjà dans les favoris
        if is_song_favorited(db, user_id, &request.item_id.id.to_string()).await? {
            return Err(Error::FavoriteAlreadyExists {
                item_type: "song".to_string(),
                item_id: request.item_id.id.to_string(),
            });
        }

        // Créer la relation avec métadonnées
        let tags = request.tags.unwrap_or_default();
        let create_sql = r#"
            RELATE $user_id->user_likes_song->$song_id SET
                tags = $tags,
                notes = $notes,
                user_rating = $user_rating,
                sort_order = $sort_order,
                is_favorite = $is_favorite,
                created_at = time::now()
        "#;

        db.query(create_sql)
            .bind(("user_id", user_thing))
            .bind(("song_id", song_thing.clone()))
            .bind(("tags", tags))
            .bind(("notes", request.notes))
            .bind(("user_rating", request.user_rating))
            .bind(("sort_order", request.sort_order.unwrap_or(0)))
            .bind(("is_favorite", request.is_favorite.unwrap_or(false)))
            .await?;

        // Récupérer la chanson avec ses métadonnées
        Self::get_song_with_metadata(db, user_id, &song_thing).await
    }

    /// Ajoute un artiste aux favoris
    pub async fn add_favorite_artist(
        db: &Surreal<Any>,
        user_id: &str,
        request: AddFavoriteRequest,
    ) -> Result<ArtistWithFavoriteMetadata, Error> {
        let user_thing = create_user_thing(user_id);
        let artist_thing = create_artist_thing(&request.item_id.id.to_string());

        // Vérifier si l'artiste existe
        if !artist_exists(db, &request.item_id.id.to_string()).await? {
            return Err(Error::ArtistNotFound {
                id: request.item_id.id.to_string().clone(),
            });
        }

        // Vérifier si l'artiste est déjà dans les favoris
        if is_artist_favorited(db, user_id, &request.item_id.id.to_string()).await? {
            return Err(Error::FavoriteAlreadyExists {
                item_type: "artist".to_string(),
                item_id: request.item_id.id.to_string(),
            });
        }

        // Créer la relation avec métadonnées
        let tags = request.tags.unwrap_or_default();
        let create_sql = r#"
            RELATE $user_id->user_likes_artist->$artist_id SET
                tags = $tags,
                notes = $notes,
                user_rating = $user_rating,
                sort_order = $sort_order,
                is_favorite = $is_favorite,
                created_at = time::now()
        "#;

        db.query(create_sql)
            .bind(("user_id", user_thing))
            .bind(("artist_id", artist_thing.clone()))
            .bind(("tags", tags))
            .bind(("notes", request.notes))
            .bind(("user_rating", request.user_rating))
            .bind(("sort_order", request.sort_order.unwrap_or(0)))
            .bind(("is_favorite", request.is_favorite.unwrap_or(false)))
            .await?;

        // Récupérer l'artiste avec ses métadonnées
        Self::get_artist_with_metadata(db, user_id, &artist_thing).await
    }

    /// Récupère les statistiques des favoris
    pub async fn get_statistics(
        db: &Surreal<Any>,
        user_id: &str,
    ) -> Result<FavoritesStatistics, Error> {
        let user_thing = create_user_thing(user_id);

        // Compter les albums
        let albums_sql =
            "SELECT count() as total FROM user_likes_album WHERE in = $user_id GROUP ALL";
        let mut albums_response = db
            .query(albums_sql)
            .bind(("user_id", user_thing.clone()))
            .await?;
        let albums_count: Option<CountResult> = albums_response.take(0)?;

        // Compter les chansons
        let songs_sql =
            "SELECT count() as total FROM user_likes_song WHERE in = $user_id GROUP ALL";
        let mut songs_response = db
            .query(songs_sql)
            .bind(("user_id", user_thing.clone()))
            .await?;
        let songs_count: Option<CountResult> = songs_response.take(0)?;

        // Compter les artistes
        let artists_sql =
            "SELECT count() as total FROM user_likes_artist WHERE in = $user_id GROUP ALL";
        let mut artists_response = db.query(artists_sql).bind(("user_id", user_thing)).await?;
        let artists_count: Option<CountResult> = artists_response.take(0)?;

        Ok(FavoritesStatistics {
            total_albums: albums_count.map(|r| r.total).unwrap_or(0),
            total_songs: songs_count.map(|r| r.total).unwrap_or(0),
            total_artists: artists_count.map(|r| r.total).unwrap_or(0),
            total_play_time: 0,         // TODO: Calculer à partir des durées
            most_played_genres: vec![], // TODO: Implémenter
            recently_added: RecentlyAddedFavorites {
                albums: vec![],
                songs: vec![],
                artists: vec![],
            },
        })
    }

    /// Récupère un album avec ses métadonnées de favori
    async fn get_album_with_metadata(
        db: &Surreal<Any>,
        user_id: &str,
        album_thing: &Thing,
    ) -> Result<AlbumWithFavoriteMetadata, Error> {
        let user_thing = create_user_thing(user_id);

        let sql = r#"
            SELECT 
                a.*,
                ula.tags,
                ula.notes,
                ula.user_rating,
                ula.sort_order,
                ula.is_favorite,
                ula.last_accessed,
                ula.created_at as favorited_at,
                (SELECT * FROM artist WHERE id IN (
                    SELECT out FROM artist_creates_album WHERE in = a.id
                )) as artists
            FROM user_likes_album ula
            JOIN album a ON ula.out = a.id
            WHERE ula.in = $user_id AND ula.out = $album_id
        "#;

        let mut response = db
            .query(sql)
            .bind(("user_id", user_thing))
            .bind(("album_id", album_thing.clone()))
            .await?;

        let album_data: Option<AlbumWithFavoriteData> = response.take(0)?;

        match album_data {
            Some(data) => Ok(AlbumWithFavoriteMetadata {
                album: AlbumWithArtists {
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
                    artists: data.artists,
                },
                favorite_metadata: Some(FavoriteMetadata {
                    tags: data.tags,
                    notes: data.notes,
                    user_rating: data.user_rating,
                    sort_order: data.sort_order,
                    is_favorite: data.is_favorite,
                    last_accessed: data.last_accessed,
                    created_at: data.favorited_at,
                }),
            }),
            None => Err(Error::FavoriteNotFound {
                item_type: "album".to_string(),
                item_id: album_thing.id.to_string(),
            }),
        }
    }

    /// Récupère une chanson avec ses métadonnées de favori
    async fn get_song_with_metadata(
        db: &Surreal<Any>,
        user_id: &str,
        song_thing: &Thing,
    ) -> Result<SongWithFavoriteMetadata, Error> {
        let user_thing = create_user_thing(user_id);

        let sql = r#"
            SELECT 
                s.*,
                uls.tags,
                uls.notes,
                uls.user_rating,
                uls.sort_order,
                uls.is_favorite,
                uls.last_accessed,
                uls.created_at as favorited_at
            FROM user_likes_song uls
            JOIN song s ON uls.out = s.id
            WHERE uls.in = $user_id AND uls.out = $song_id
        "#;

        let mut response = db
            .query(sql)
            .bind(("user_id", user_thing))
            .bind(("song_id", song_thing.clone()))
            .await?;

        let song_data: Option<SongWithFavoriteData> = response.take(0)?;

        match song_data {
            Some(data) => Ok(SongWithFavoriteMetadata {
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
                favorite_metadata: Some(FavoriteMetadata {
                    tags: data.tags,
                    notes: data.notes,
                    user_rating: data.user_rating,
                    sort_order: data.sort_order,
                    is_favorite: data.is_favorite,
                    last_accessed: data.last_accessed,
                    created_at: data.favorited_at,
                }),
            }),
            None => Err(Error::FavoriteNotFound {
                item_type: "song".to_string(),
                item_id: song_thing.id.to_string(),
            }),
        }
    }

    /// Récupère un artiste avec ses métadonnées de favori
    async fn get_artist_with_metadata(
        db: &Surreal<Any>,
        user_id: &str,
        artist_thing: &Thing,
    ) -> Result<ArtistWithFavoriteMetadata, Error> {
        let user_thing = create_user_thing(user_id);

        let sql = r#"
            SELECT 
                a.*,
                ular.tags,
                ular.notes,
                ular.user_rating,
                ular.sort_order,
                ular.is_favorite,
                ular.last_accessed,
                ular.created_at as favorited_at
            FROM user_likes_artist ular
            JOIN artist a ON ular.out = a.id
            WHERE ular.in = $user_id AND ular.out = $artist_id
        "#;

        let mut response = db
            .query(sql)
            .bind(("user_id", user_thing))
            .bind(("artist_id", artist_thing.clone()))
            .await?;

        let artist_data: Option<ArtistWithFavoriteData> = response.take(0)?;

        match artist_data {
            Some(data) => Ok(ArtistWithFavoriteMetadata {
                artist: Artist {
                    id: data.artist.id,
                    name: data.artist.name,
                    genres: data.artist.genres,
                    country_code: data.artist.country_code,
                    artist_image: data.artist.artist_image,
                    albums_count: data.artist.albums_count,
                    songs_count: data.artist.songs_count,
                },
                favorite_metadata: Some(FavoriteMetadata {
                    tags: data.tags,
                    notes: data.notes,
                    user_rating: data.user_rating,
                    sort_order: data.sort_order,
                    is_favorite: data.is_favorite,
                    last_accessed: data.last_accessed,
                    created_at: data.favorited_at,
                }),
            }),
            None => Err(Error::FavoriteNotFound {
                item_type: "artist".to_string(),
                item_id: artist_thing.id.to_string(),
            }),
        }
    }
}

// Structures internes pour les requêtes
#[derive(Debug, Deserialize)]
struct AlbumWithFavoriteData {
    album: AlbumData,
    tags: Vec<String>,
    notes: Option<String>,
    user_rating: Option<u8>,
    sort_order: i32,
    is_favorite: bool,
    last_accessed: Option<Datetime>,
    favorited_at: Datetime,
    artists: Vec<Artist>,
}

#[derive(Debug, Deserialize)]
struct AlbumData {
    id: Option<Thing>,
    title: String,
    cover_url: Option<String>,
    release_year: Option<u16>,
    genres: Vec<String>,
    langs: Vec<String>,
    dominant_color: Option<String>,
    total_tracks: u32,
    total_duration: surrealdb::sql::Duration,
    total_listens: u32,
    total_user_listens: u32,
    total_likes: u32,
}

#[derive(Debug, Deserialize)]
struct SongWithFavoriteData {
    song: SongData,
    tags: Vec<String>,
    notes: Option<String>,
    user_rating: Option<u8>,
    sort_order: i32,
    is_favorite: bool,
    last_accessed: Option<Datetime>,
    favorited_at: Datetime,
}

#[derive(Debug, Deserialize)]
struct SongData {
    id: Option<Thing>,
    title: String,
    file_url: String,
    duration: surrealdb::sql::Duration,
    song_index: u16,
    tempo: f32,
    total_listens: u32,
    total_user_listens: u32,
    total_likes: u32,
}

#[derive(Debug, Deserialize)]
struct ArtistWithFavoriteData {
    artist: ArtistData,
    tags: Vec<String>,
    notes: Option<String>,
    user_rating: Option<u8>,
    sort_order: i32,
    is_favorite: bool,
    last_accessed: Option<Datetime>,
    favorited_at: Datetime,
}

#[derive(Debug, Deserialize)]
struct ArtistData {
    id: Option<Thing>,
    name: String,
    genres: Vec<crate::models::music_genre::MusicGenre>,
    country_code: String,
    artist_image: Option<String>,
    albums_count: u16,
    songs_count: u16,
}
