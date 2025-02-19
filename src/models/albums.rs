use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Deserialize)]
pub struct Album {
    pub id: Thing,
    pub title: String,
    pub artist: Vec<String>,
    #[serde(skip)]
    pub songs: Vec<Song>,
    pub cover: String,
    // #[serde(rename = "cover_avif")]
    pub cover_avif: String,
    pub lang: String,
    pub genre: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAllAlbumsResponse {
    #[serde(rename = "_id")]
    pub id: String,
    pub title: String,
    pub artist: Vec<String>,
    pub cover: String,
    // #[serde(rename = "cover_avif")]
    pub cover_avif: String,
    pub lang: String,
    pub genre: Vec<String>,
    pub song_length: usize,
}

impl From<Album> for GetAllAlbumsResponse {
    fn from(album: Album) -> Self {
        Self {
            id: format!("{}:{}", album.id.tb, album.id.id),
            title: album.title,
            artist: album.artist,
            cover: album.cover,
            cover_avif: album.cover_avif,
            lang: album.lang,
            genre: album.genre,
            song_length: album.songs.len(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Song {
    pub title: String,
    pub file: String,
}
