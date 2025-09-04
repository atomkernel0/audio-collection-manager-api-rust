use crate::error::Error;
use crate::models::album::AlbumsMetaResponse;
use crate::{
    helpers::{
        album_helpers::album_exists,
        thing_helpers::{create_album_thing},
    },
    models::{
        album::{AlbumWithArtists, AlbumWithRelations},
        database_helpers::{CountResult, RelationId},
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

    /// Fetches the initial albums with metadata for optimized SSR
    /// Makes 2 parallel requests: one for the albums, one for the count
    pub async fn get_initial_albums_with_meta(
        db: &Surreal<Any>,
        limit: u32,
    ) -> Result<AlbumsMetaResponse, Error> {
        let safe_limit = limit.min(50);
        
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

    /// Fetches a batch of albums for progressive client-side loading
    pub async fn get_albums_batch(
        db: &Surreal<Any>,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<AlbumWithArtists>, Error> {
        // Limite de sécurité par batch (max 100)
        let safe_limit = limit.min(100);
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
        let safe_limit = limit.min(100);
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
        
        // Déterminer l'ordre de tri
        let order_clause = match sort_by.as_deref() {
            Some("popular") => "ORDER BY total_listens DESC",
            Some("recent") => "ORDER BY release_year DESC NULLS LAST",
            Some("alphabetical") | _ => "ORDER BY title ASC",
        };
        
        // Requête pour les albums filtrés
        let albums_query = format!(
            "SELECT *, 
             <-artist_creates_album<-artist.* AS artists 
             FROM album 
             {} 
             {} 
             START {} LIMIT {};",
            where_clause, order_clause, safe_offset, safe_limit
        );
        
        // Requête pour le count total avec les mêmes filtres
        let count_query = format!(
            "SELECT count() AS total 
             FROM album 
             {} 
             GROUP ALL;",
            where_clause
        );
        
        // Créer le contexte de binding pour les requêtes
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
        
        // Exécuter les requêtes en parallèle
        let (albums_result, count_result) = tokio::join!(
            albums_query_builder,
            count_query_builder
        );
        
        // Traiter les résultats
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
        // Utiliser directement les strings formatées au lieu des Thing
        let album_record = format!("album:{}", album_id);

        if !album_exists(db, &album_id).await? {
            return Err(Error::AlbumNotFound {
                id: album_id.to_string(),
            });
        }

        if let Some(user_id) = user_id {
            let user_record = format!("user:{}", user_id);

            let check_query = "
            SELECT count() as total 
            FROM user_listens_album 
            WHERE in = type::thing($user_id) 
            AND out = type::thing($album_id) 
            GROUP ALL
        ";

            let mut check_response = db
                .query(check_query)
                .bind(("user_id", user_record.clone()))
                .bind(("album_id", album_record.clone()))
                .await?;

            let count_result: Option<CountResult> = check_response.take(0)?;
            let exists = count_result.map(|r| r.total > 0).unwrap_or(false);

            if exists {
                let get_id_query = "
                SELECT id 
                FROM user_listens_album 
                WHERE in = type::thing($user_id) 
                AND out = type::thing($album_id) 
                LIMIT 1
            ";

                let mut id_response = db
                    .query(get_id_query)
                    .bind(("user_id", user_record.clone()))
                    .bind(("album_id", album_record.clone()))
                    .await?;

                let existing_relation: Option<RelationId> = id_response.take(0)?;

                if let Some(relation) = existing_relation {
                    let relation_id_str = relation.id.to_string();

                    let update_query = format!(
                    "LET $album_duration = (SELECT VALUE total_duration FROM type::thing($album_id) LIMIT 1)[0] OR 0s;
                     UPDATE {} SET
                         total_listens += 1,
                         total_duration = total_duration + $album_duration,
                         recent_dates = array::slice(array::prepend(recent_dates, time::now()), 0, 30),
                         last_listened_at = time::now();",
                    relation_id_str
                );

                    let _ = db
                        .query(update_query)
                        .bind(("album_id", album_record.clone()))
                        .await?;
                } else {
                    return Err(Error::DbError("No existing relation found".to_string()));
                }
            } else {
                let create_query = "
                LET $album_duration = (SELECT VALUE total_duration FROM type::thing($album_id) LIMIT 1)[0] OR 0s;
                CREATE user_listens_album CONTENT {
                    in: type::thing($user_id),
                    out: type::thing($album_id),
                    total_listens: 1,
                    total_duration: $album_duration,
                    recent_dates: [time::now()],
                    first_listened_at: time::now(),
                    last_listened_at: time::now()
                };
            ";

                let mut create_response = db
                    .query(create_query)
                    .bind(("user_id", user_record))
                    .bind(("album_id", album_record.clone()))
                    .await?;

                let create_result: Vec<serde_json::Value> = create_response.take(1)?;

                if create_result.is_empty() {
                    return Err(Error::DbError(
                        "Failed to create listen relation".to_string(),
                    ));
                }
            }
        }

        // Mise à jour du compteur global
        let update_query =
            "UPDATE type::thing($album_id) SET total_listens = (total_listens OR 0) + 1";
        let _ = db
            .query(update_query)
            .bind(("album_id", album_record))
            .await?;

        Ok(true)
    }
}
