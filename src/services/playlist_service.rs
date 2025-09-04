use chrono::Utc;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use surrealdb::{engine::any::Any, Surreal};

use crate::helpers::song_helpers::song_exists;
use crate::models::playlist::PlaylistWithSongs;
use crate::{
    helpers::thing_helpers::{create_playlist_thing, create_song_thing, create_user_thing},
    models::playlist::{CreatePlaylistRequest, Playlist},
    Error,
};

#[derive(Serialize, Deserialize)]
struct UserLikesPlaylist {
    id: Thing,
    #[serde(rename = "in")]
    user_id: Thing,
    #[serde(rename = "out")]
    playlist_id: Thing,
    created_at: surrealdb::sql::Datetime,
    tags: Vec<String>,
    notes: Option<String>,
    user_rating: Option<i32>,
    sort_order: i64,
    is_favorite: bool,
    last_accessed: Option<surrealdb::sql::Datetime>,
}

pub struct PlaylistService;

impl PlaylistService {
    /// Fonction utilitaire pour valider l'existence et la propriété d'une playlist
    async fn validate_playlist_ownership(
        db: &Surreal<Any>,
        playlist_id: &str,
        user_id: &str,
    ) -> Result<Playlist, Error> {
        let user_thing = create_user_thing(user_id);
        let playlist_thing = create_playlist_thing(playlist_id);

        let mut playlists: Vec<Playlist> = db
            .query("SELECT * FROM playlist WHERE id = $playlist AND created_by = $user")
            .bind(("playlist", playlist_thing))
            .bind(("user", user_thing))
            .await
            .map_err(|e| {
                Error::DbError(format!(
                    "Erreur lors de la validation de la playlist: {}",
                    e
                ))
            })?
            .take(0)
            .map_err(|e| Error::DbError(format!("Erreur de désérialisation playlist: {}", e)))?;

        playlists.pop().ok_or_else(|| Error::PlaylistNotFound {
            id: format!(
                "Playlist '{}' non trouvée ou non autorisée pour l'utilisateur '{}'",
                playlist_id, user_id
            ),
        })
    }

    /// Fonction utilitaire pour vérifier si une chanson existe déjà dans une playlist
    async fn song_exists_in_playlist(
        db: &Surreal<Any>,
        playlist_id: &str,
        song_id: &str,
    ) -> Result<bool, Error> {
        let playlist_thing = create_playlist_thing(playlist_id);
        let song_thing = create_song_thing(song_id);

        #[derive(serde::Deserialize)]
        struct Count {
            count: u32,
        }

        let mut result: Vec<Count> = db
            .query("SELECT count() FROM playlist_contains_song WHERE in = $playlist AND out = $song GROUP ALL")
            .bind(("playlist", playlist_thing))
            .bind(("song", song_thing))
            .await
            .map_err(|e| Error::DbError(format!("Erreur lors de la vérification de duplication: {}", e)))?
            .take(0)
            .map_err(|e| Error::DbError(format!("Erreur de désérialisation duplication: {}", e)))?;

        Ok(result.pop().map(|c| c.count).unwrap_or(0) > 0)
    }

    /// Crée une nouvelle playlist pour un utilisateur
    pub async fn create_playlist(
        db: &Surreal<Any>,
        user_id: &str,
        playlist: CreatePlaylistRequest,
    ) -> Result<Thing, Error> {
        let user_thing = create_user_thing(user_id);

        // Validation basique
        if playlist.name.trim().is_empty() {
            return Err(Error::InvalidInput {
                reason: "Le nom de la playlist ne peut pas être vide".to_string(),
            });
        }

        let now = Utc::now();

        // Créer la playlist
        let query = r#"
        CREATE playlist SET
            name = $name,
            cover_url = $cover_url,
            is_public = $is_public,
            created_by = $created_by,
            created_at = $created_at,
            updated_at = $updated_at,
            songs_count = 0,
            total_duration = 0s,
            total_listens = 0,
            total_likes = 0
        RETURN id
    "#;

        #[derive(serde::Deserialize, Debug)]
        struct Record {
            id: Thing,
        }

        let mut created_records: Vec<Record> = db
            .query(query)
            .bind(("name", playlist.name))
            .bind(("cover_url", playlist.cover_url))
            .bind(("is_public", playlist.is_public))
            .bind(("created_by", user_thing.clone()))
            .bind(("created_at", surrealdb::sql::Datetime::from(now)))
            .bind(("updated_at", surrealdb::sql::Datetime::from(now)))
            .await?
            .take(0)?;

        let playlist_thing = created_records
            .pop()
            .ok_or_else(|| Error::DbError("Erreur lors de la création de la playlist".to_string()))?
            .id;

        // Créer la relation user -> playlist
        let _ = db
            .query("RELATE $user->user_creates_playlist->$playlist")
            .bind(("user", user_thing))
            .bind(("playlist", playlist_thing.clone()))
            .await?;

        Ok(playlist_thing)
    }

    pub async fn add_song_to_playlist(
        db: &Surreal<Any>,
        user_id: &str,
        song_id: &str,
        playlist_id: &str,
    ) -> Result<(), Error> {
        if user_id.trim().is_empty() {
            return Err(Error::InvalidInput {
                reason: "L'ID utilisateur ne peut pas être vide".to_string(),
            });
        }
        if song_id.trim().is_empty() {
            return Err(Error::InvalidInput {
                reason: "L'ID de la chanson ne peut pas être vide".to_string(),
            });
        }
        if playlist_id.trim().is_empty() {
            return Err(Error::InvalidInput {
                reason: "L'ID de la playlist ne peut pas être vide".to_string(),
            });
        }

        let playlist = Self::validate_playlist_ownership(db, playlist_id, user_id).await?;

        if playlist.songs_count >= 1000 {
            return Err(Error::InvalidInput {
                reason: format!(
                    "La playlist '{}' a atteint la limite maximale de 1000 chansons",
                    playlist_id
                ),
            });
        }

        let song_check = song_exists(db, song_id).await.map_err(|e| {
            Error::DbError(format!(
                "Erreur lors de la vérification de la chanson '{}': {}",
                song_id, e
            ))
        })?;

        if !song_check {
            return Err(Error::SongNotFound {
                id: song_id.to_string(),
            });
        }

        if Self::song_exists_in_playlist(db, playlist_id, song_id).await? {
            return Err(Error::SongAlreadyExistsInPlaylist {
                song_id: song_id.to_string(),
                playlist_id: playlist_id.to_string(),
            });
        }

        let user_thing = create_user_thing(user_id);
        let song_thing = create_song_thing(song_id);
        let playlist_thing = create_playlist_thing(playlist_id);

        let _: surrealdb::Response = db
            .query("RELATE $playlist->playlist_contains_song->$song SET added_at = $added_at, added_by = $added_by")
            .bind(("playlist", playlist_thing))
            .bind(("song", song_thing))
            .bind(("added_at", surrealdb::sql::Datetime::from(Utc::now())))
            .bind(("added_by", user_thing))
            .await
            .map_err(|e| Error::DbError(format!("Erreur lors de l'ajout de la chanson '{}' à la playlist '{}': {}", song_id, playlist_id, e)))?;

        Self::update_playlist_stats(db, playlist_id)
            .await
            .map_err(|e| {
                Error::DbError(format!(
                    "Erreur lors de la mise à jour des statistiques de la playlist '{}': {}",
                    playlist_id, e
                ))
            })?;

        Ok(())
    }

    /// Récupère toutes les playlists d'un utilisateur
    pub async fn get_user_playlists(
        db: &Surreal<Any>,
        user_id: &str,
    ) -> Result<Vec<Playlist>, Error> {
        let user_thing = create_user_thing(user_id);

        let playlists: Vec<Playlist> = db
            .query(
                r#"
            SELECT *
            FROM playlist
            WHERE created_by = $user
            ORDER BY created_at DESC
        "#,
            )
            .bind(("user", user_thing))
            .await?
            .take(0)?;

        Ok(playlists)
    }

    /// Récupère une playlist spécifique avec ses chansons
    pub async fn get_playlist_with_songs(
        db: &Surreal<Any>,
        playlist_id: &str,
    ) -> Result<PlaylistWithSongs, Error> {
        let playlist_thing = create_playlist_thing(playlist_id);

        println!("DEBUG: Fetching playlist with songs for playlist_id: {}", playlist_id);

        let mut playlists: Vec<PlaylistWithSongs> = db
            .query(
                r#"
                SELECT *,
                    (SELECT
                        out.id as id,
                        out.title as title,
                        out.duration OR 0s as duration,
                        out.file_url as file_url,
                        out.song_index as song_index,
                        out.tempo as tempo,
                        out.total_listens as total_listens,
                        out.total_user_listens as total_user_listens,
                        out.total_likes as total_likes,
                        added_at,
                        (out<-artist_performs_song<-artist) AS artists,
                        (out<-album_contains_song<-album)[0] AS album
                    FROM playlist_contains_song WHERE in = $parent.id AND out.id IS NOT NONE ORDER BY added_at ASC) AS songs
                FROM playlist
                WHERE id = $playlist
                FETCH created_by, songs, songs.artists, songs.album
            "#,
            )
            .bind(("playlist", playlist_thing))
            .await
            .map_err(|e| {
                eprintln!("DEBUG: Error fetching playlist with songs: {}", e);
                Error::DbError(format!("Erreur lors de la récupération de la playlist: {}", e))
            })?
            .take(0)
            .map_err(|e| {
                eprintln!("DEBUG: Deserialization error in get_playlist_with_songs: {}", e);
                Error::DbError(format!("Erreur de désérialisation playlist: {}", e))
            })?;

        playlists.pop().ok_or_else(|| Error::PlaylistNotFound {
            id: format!("Playlist '{}' non trouvée", playlist_id),
        })
    }

    /// Supprime une chanson d'une playlist
    pub async fn remove_song_from_playlist(
        db: &Surreal<Any>,
        user_id: &str,
        song_id: &str,
        playlist_id: &str,
    ) -> Result<(), Error> {
        let user_thing = create_user_thing(user_id);
        let song_thing = create_song_thing(song_id);
        let playlist_thing = create_playlist_thing(playlist_id);

        // Vérifier que la playlist appartient à l'utilisateur
        let playlist_check: Option<Playlist> = db
            .query("SELECT * FROM playlist WHERE id = $playlist AND created_by = $user")
            .bind(("playlist", playlist_thing.clone()))
            .bind(("user", user_thing))
            .await?
            .take(0)?;

        if playlist_check.is_none() {
            return Err(Error::PlaylistNotFound {
                id: "Playlist non trouvée ou non autorisée".to_string(),
            });
        }

        // Supprimer la relation
        let _ = db
            .query("DELETE FROM playlist_contains_song WHERE in = $playlist AND out = $song")
            .bind(("playlist", playlist_thing.clone()))
            .bind(("song", song_thing))
            .await?;

        // Mettre à jour les statistiques
        Self::update_playlist_stats(db, playlist_id).await?;

        Ok(())
    }

    /// Supprime une playlist
    pub async fn delete_playlist(
        db: &Surreal<Any>,
        user_id: &str,
        playlist_id: &str,
    ) -> Result<(), Error> {
        let user_thing = create_user_thing(user_id);
        let playlist_thing = create_playlist_thing(playlist_id);

        // Vérifier que la playlist appartient à l'utilisateur
        let playlist_check: Option<Playlist> = db
            .query("SELECT * FROM playlist WHERE id = $playlist AND created_by = $user")
            .bind(("playlist", playlist_thing.clone()))
            .bind(("user", user_thing))
            .await?
            .take(0)?;

        if playlist_check.is_none() {
            return Err(Error::PlaylistNotFound {
                id: "Playlist non trouvée ou non autorisée".to_string(),
            });
        }

        // Supprimer la playlist et toutes ses relations
        let _ = db
            .query("DELETE FROM playlist WHERE id = $playlist")
            .bind(("playlist", playlist_thing))
            .await?;

        Ok(())
    }

    /// Met à jour les statistiques d'une playlist
    async fn update_playlist_stats(db: &Surreal<Any>, playlist_id: &str) -> Result<(), Error> {
        let playlist_thing = create_playlist_thing(playlist_id);

        // Compter le nombre de chansons dans la playlist
        // Utiliser une structure pour désérialiser le résultat du count
        #[derive(serde::Deserialize)]
        struct CountResult {
            count: i64,
        }

        let mut count_results: Vec<CountResult> = db
            .query("SELECT count() AS count FROM playlist_contains_song WHERE in = $playlist GROUP ALL")
            .bind(("playlist", playlist_thing.clone()))
            .await
            .map_err(|e| Error::DbError(format!("Erreur lors du comptage des chansons: {}", e)))?
            .take(0)
            .map_err(|e| Error::DbError(format!("Erreur de désérialisation du comptage: {}", e)))?;

        let songs_count = count_results.pop().map(|r| r.count).unwrap_or(0);

        // Récupérer toutes les durées et les additionner
        #[derive(serde::Deserialize)]
        struct SongDuration {
            duration: Option<surrealdb::sql::Duration>,
        }

        let query = "SELECT out.duration as duration FROM playlist_contains_song WHERE in = $playlist AND out.id IS NOT NONE";

        println!("DEBUG: Fetching song durations for playlist_id: {}", playlist_id);

        let duration_results: Vec<SongDuration> = db
            .query(query)
            .bind(("playlist", playlist_thing.clone()))
            .await
            .map_err(|e| {
                eprintln!("DEBUG: Error fetching durations: {}", e);
                Error::DbError(format!("Erreur lors de la récupération des durées: {}", e))
            })?
            .take(0)
            .map_err(|e| {
                eprintln!("DEBUG: Deserialization error for durations: {}", e);
                Error::DbError(format!("Erreur de désérialisation des durées: {}", e))
            })?;

        let total_duration = duration_results
            .into_iter()
            .filter_map(|r| r.duration)
            .fold(surrealdb::sql::Duration::from_secs(0), |acc, d| {
                surrealdb::sql::Duration::from_secs(acc.as_secs() + d.as_secs())
            });

        // Mettre à jour la playlist avec les nouvelles statistiques
        let _: surrealdb::Response = db
            .query("UPDATE playlist SET songs_count = $songs_count, total_duration = $total_duration, updated_at = $updated_at WHERE id = $playlist")
            .bind(("songs_count", songs_count))
            .bind(("total_duration", total_duration))
            .bind(("updated_at", surrealdb::sql::Datetime::from(Utc::now())))
            .bind(("playlist", playlist_thing))
            .await
            .map_err(|e| Error::DbError(format!("Erreur lors de la mise à jour des statistiques: {}", e)))?;

        Ok(())
    }

    /// Récupère les playlists publiques
    pub async fn get_public_playlists(db: &Surreal<Any>) -> Result<Vec<Playlist>, Error> {
        let playlists: Vec<Playlist> = db
            .query(
                r#"
            SELECT *
            FROM playlist
            WHERE is_public = true
            ORDER BY created_at DESC
        "#,
            )
            .await?
            .take(0)?;

        Ok(playlists)
    }

    /// Like/Unlike une playlist
    pub async fn toggle_playlist_like(
        db: &Surreal<Any>,
        user_id: &str,
        playlist_id: &str,
    ) -> Result<bool, Error> {
        let user_thing = create_user_thing(user_id);
        let playlist_thing = create_playlist_thing(playlist_id);

        // Vérifier si l'utilisateur a déjà liké la playlist
        let sql_check =
            "SELECT * FROM user_likes_playlist WHERE in = $user AND out = $playlist LIMIT 1";

        let mut response = db
            .query(sql_check)
            .bind(("user", user_thing.clone()))
            .bind(("playlist", playlist_thing.clone()))
            .await?;

        let result: Vec<UserLikesPlaylist> = response.take(0)?;
        let exists = !result.is_empty();

        if exists {
            // Unlike - supprimer le like existant
            let sql_delete = "DELETE user_likes_playlist WHERE in = $user AND out = $playlist";
            let _response = db
                .query(sql_delete)
                .bind(("user", user_thing))
                .bind(("playlist", playlist_thing.clone()))
                .await?;

            // Pas besoin de traiter le résultat de la suppression
            Ok(false)
        } else {
            // Like - créer un nouveau like
            let sql_create =
                "RELATE $user->user_likes_playlist->$playlist SET created_at = time::now()";
            db.query(sql_create)
                .bind(("user", user_thing))
                .bind(("playlist", playlist_thing.clone()))
                .await?
                .check()?;

            Ok(true)
        }
    }
}
