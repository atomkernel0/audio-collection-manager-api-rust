use crate::error::Result;
use serde::{Deserialize, Serialize};
use surrealdb::{engine::any::Any, Surreal};

use crate::models::{album::AlbumWithArtists, artist::Artist, song::SongWithRelations};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub albums: Vec<AlbumWithArtists>,
    pub artists: Vec<Artist>,
    pub songs: Vec<SongWithRelations>,
}

pub struct SearchService;

impl SearchService {
    pub async fn search_albums_songs_artists(
        db: &Surreal<Any>,
        term: &str,
    ) -> Result<SearchResult> {
        let albums: Vec<AlbumWithArtists> = Self::search_albums(db, term).await?;
        let songs: Vec<SongWithRelations> = Self::search_songs(db, term).await?;
        let artists: Vec<Artist> = Self::search_artists(db, term).await?;

        Ok(SearchResult {
            albums,
            artists,
            songs,
        })
    }

    async fn search_albums(db: &Surreal<Any>, term: &str) -> Result<Vec<AlbumWithArtists>> {
        let sql = "
            SELECT *, <-artist_creates_album<-artist.* AS artists
            FROM album
            WHERE string::lowercase(title) CONTAINS string::lowercase($term);
        ";

        let mut res = db.query(sql).bind(("term", term.to_string())).await?;
        let albums: Vec<AlbumWithArtists> = res.take(0)?;
        Ok(albums)
    }

    async fn search_songs(db: &Surreal<Any>, term: &str) -> Result<Vec<SongWithRelations>> {
        let sql = "SELECT *,
            (SELECT * FROM <-artist_performs_song<-artist) AS artists,
            (SELECT * FROM <-album_contains_song<-album)[0] AS album
        FROM song
        WHERE string::lowercase(title) CONTAINS string::lowercase($term);";

        let mut res = db.query(sql).bind(("term", term.to_string())).await?;
        let songs: Vec<SongWithRelations> = res.take(0)?;
        Ok(songs)
    }

    async fn search_artists(db: &Surreal<Any>, term: &str) -> Result<Vec<Artist>> {
        let sql = "
            SELECT *
            FROM artist
            WHERE string::lowercase(name) CONTAINS string::lowercase($term);
        ";
        let mut res = db.query(sql).bind(("term", term.to_string())).await?;
        let artists: Vec<Artist> = res.take(0)?;
        Ok(artists)
    }
}
