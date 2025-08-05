use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use crate::models::{
    album::{Album, AlbumWithArtists},
    music_genre::MusicGenre,
    song::Song,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artist {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,

    pub name: String,
    pub genres: Vec<MusicGenre>,
    pub country_code: String,
    pub artist_image: Option<String>,
    #[serde(default)]
    pub albums_count: u16,
    #[serde(default)]
    pub songs_count: u16,
    #[serde(default)]
    pub total_likes: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtistWithAlbums {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub name: String,
    pub genres: Vec<MusicGenre>,
    pub country_code: String,
    pub artist_image: Option<String>,
    #[serde(default)]
    pub albums_count: u16,
    #[serde(default)]
    pub songs_count: u16,
    #[serde(default)]
    pub total_likes: u32,
    pub albums: Vec<Album>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtistWithAlbumsAndTopSongs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub name: String,
    pub genres: Vec<MusicGenre>,
    pub country_code: String,
    pub artist_image: Option<String>,
    #[serde(default)]
    pub albums_count: u16,
    #[serde(default)]
    pub songs_count: u16,
    #[serde(default)]
    pub total_likes: u32,
    pub albums: Vec<AlbumWithArtists>,
    pub top_songs: Vec<Song>,
}
