use crate::{error::Result, Error};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumSong {
    pub title: String,
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum MusicGenre {
    Rac,
    Rif,
    Nsbm,
    Oi,
    Rap,
    Thrash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub id: Option<Thing>,
    pub title: String,
    pub artist: Vec<String>,
    pub songs: Vec<AlbumSong>,
    pub cover: String,
    pub cover_avif: String,
    pub lang: String,
    pub genre: Vec<MusicGenre>,
}

#[derive(Clone)]
pub struct AlbumModelController {
    db: surrealdb::Surreal<surrealdb::engine::local::Db>,
}

impl AlbumModelController {
    pub async fn new() -> Result<Self> {
        tokio::fs::create_dir_all("data")
            .await
            .map_err(|e| Error::DbError(e.to_string()))?;

        let db = surrealdb::Surreal::new::<surrealdb::engine::local::File>("data")
            .await
            .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

        db.use_ns("music")
            .use_db("albums")
            .await
            .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

        Ok(Self { db })
    }

    pub async fn get_all_albums(&self) -> Result<Vec<Album>> {
        let albums: Vec<Album> = self
            .db
            .select("album")
            .await
            .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

        Ok(albums)
    }

    pub async fn get_album_by_id(&self, id: &str) -> Result<Option<Album>> {
        let album: Option<Album> = self
            .db
            .select(("album", id))
            .await
            .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

        Ok(album)
    }
}
