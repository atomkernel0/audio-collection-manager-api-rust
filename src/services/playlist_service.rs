use chrono::Utc;
use surrealdb::sql::{Thing, Value};
use surrealdb::{engine::any::Any, Surreal};

use crate::helpers::song_helpers::song_exists;
use crate::models::playlist::PlaylistWithSongs;
use crate::{
    helpers::thing_helpers::{create_playlist_thing, create_song_thing, create_user_thing},
    models::playlist::{CreatePlaylistRequest, Playlist},
    Error,
};

/*
    implémentation Angular:
    bouton "ajouter dans playlist" => affiche la liste des playlists
    créer -> affiche Dialog avec "indiquez le nom de la playlist"
    ensuite ça affiche la liste avec la nouvelle, et on ajoute !
*/

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

    /// Ajoute une chanson à une playlist avec validation complète et gestion d'erreurs robuste
    pub async fn add_song_to_playlist(
        db: &Surreal<Any>,
        user_id: &str,
        song_id: &str,
        playlist_id: &str,
    ) -> Result<(), Error> {
        // Validation des paramètres d'entrée
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

        // Vérifier que la playlist appartient à l'utilisateur
        let playlist = Self::validate_playlist_ownership(db, playlist_id, user_id).await?;

        // Vérification optionnelle : limite du nombre de chansons par playlist
        if playlist.songs_count >= 1000 {
            return Err(Error::InvalidInput {
                reason: format!(
                    "La playlist '{}' a atteint la limite maximale de 1000 chansons",
                    playlist_id
                ),
            });
        }

        // Vérifier que la chanson existe
        let song_check = song_exists(db, song_id).await.map_err(|e| {
            Error::DbError(format!(
                "Erreur lors de la vérification de la chanson '{}': {}",
                song_id, e
            ))
        })?;

        if !song_check {
            return Err(Error::SongNotFound {
                id: format!("Chanson '{}' non trouvée", song_id),
            });
        }

        // Vérifier que la chanson n'est pas déjà dans la playlist
        if Self::song_exists_in_playlist(db, playlist_id, song_id).await? {
            return Err(Error::SongAlreadyExists {
                id: format!(
                    "La chanson '{}' est déjà dans la playlist '{}'",
                    song_id, playlist_id
                ),
            });
        }

        let user_thing = create_user_thing(user_id);
        let song_thing = create_song_thing(song_id);
        let playlist_thing = create_playlist_thing(playlist_id);

        // Utiliser RELATE pour créer la relation (syntaxe SurrealDB recommandée)
        // Le résultat de RELATE est un enregistrement de relation, pas besoin de le désérialiser
        let _: surrealdb::Response = db
            .query("RELATE $playlist->playlist_contains_song->$song SET added_at = $added_at, added_by = $added_by")
            .bind(("playlist", playlist_thing))
            .bind(("song", song_thing))
            .bind(("added_at", surrealdb::sql::Datetime::from(Utc::now())))
            .bind(("added_by", user_thing))
            .await
            .map_err(|e| Error::DbError(format!("Erreur lors de l'ajout de la chanson '{}' à la playlist '{}': {}", song_id, playlist_id, e)))?;

        // Mettre à jour les statistiques de la playlist
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

        let mut playlists: Vec<Playlist> = db
            .query(
                r#"
            SELECT *
            FROM playlist
            WHERE id = $playlist
        "#,
            )
            .bind(("playlist", playlist_thing.clone()))
            .await?
            .take(0)?;

        let playlist = playlists.pop().ok_or_else(|| Error::PlaylistNotFound {
            id: "Playlist non trouvée".to_string(),
        })?;

        // Récupérer l'utilisateur créateur
        let mut users: Vec<crate::models::user::User> = db
            .query("SELECT * FROM user WHERE id = $user_id")
            .bind(("user_id", playlist.created_by.clone()))
            .await?
            .take(0)?;

        let created_by = users.pop().ok_or_else(|| Error::DbError {
            0: "Utilisateur créateur non trouvé".to_string(),
        })?;

        // Récupérer les chansons de la playlist
        let songs: Vec<crate::models::song::Song> = db
            .query(
                r#"
            SELECT song.*
            FROM playlist_contains_song
            WHERE in = $playlist
            ORDER BY added_at ASC
            FETCH out
        "#,
            )
            .bind(("playlist", playlist_thing))
            .await?
            .take(0)?;

        let playlist_with_songs = PlaylistWithSongs {
            id: playlist.id,
            name: playlist.name,
            cover_url: playlist.cover_url,
            dominant_color: playlist.dominant_color,
            is_public: playlist.is_public,
            created_at: playlist.created_at,
            updated_at: playlist.updated_at,
            songs_count: playlist.songs_count,
            total_duration: playlist.total_duration,
            total_listens: playlist.total_listens,
            total_likes: playlist.total_likes,
            created_by,
            songs,
        };

        Ok(playlist_with_songs)
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

        // Calculer la durée totale (optionnelle car peut être NULL si aucune chanson)
        #[derive(serde::Deserialize)]
        struct DurationResult {
            total: Option<surrealdb::sql::Duration>,
        }

        let mut duration_results: Vec<DurationResult> = db
            .query("SELECT math::sum(out.duration) AS total FROM playlist_contains_song WHERE in = $playlist GROUP ALL")
            .bind(("playlist", playlist_thing.clone()))
            .await
            .map_err(|e| Error::DbError(format!("Erreur lors du calcul de la durée: {}", e)))?
            .take(0)
            .map_err(|e| Error::DbError(format!("Erreur de désérialisation de la durée: {}", e)))?;

        let total_duration = duration_results
            .pop()
            .and_then(|r| r.total)
            .unwrap_or(surrealdb::sql::Duration::from_secs(0));

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
        let existing: Option<Value> = db
            .query("SELECT * FROM user_likes_playlist WHERE in = $user AND out = $playlist")
            .bind(("user", user_thing.clone()))
            .bind(("playlist", playlist_thing.clone()))
            .await?
            .take(0)?;

        if existing.is_some() {
            // Unlike
            let _ = db
                .query("DELETE FROM user_likes_playlist WHERE in = $user AND out = $playlist")
                .bind(("user", user_thing))
                .bind(("playlist", playlist_thing))
                .await?;
            Ok(false)
        } else {
            // Like
            let _ = db
                .query("RELATE $user->user_likes_playlist->$playlist SET created_at = $created_at")
                .bind(("user", user_thing))
                .bind(("playlist", playlist_thing))
                .bind(("created_at", surrealdb::sql::Datetime::from(Utc::now())))
                .await?;
            Ok(true)
        }
    }
}
