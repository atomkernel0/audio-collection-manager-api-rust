use serde::{Deserialize, Serialize};
use surrealdb::sql::{Duration, Thing};

use crate::models::{artist::Artist, song::Song};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Album {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,

    pub title: String,
    pub cover_url: Option<String>,
    pub release_year: Option<u16>,
    pub genres: Vec<String>,
    pub langs: Vec<String>,
    pub dominant_color: Option<String>,
    pub total_tracks: u32,
    pub total_duration: Duration,

    // Stats
    #[serde(default)]
    pub total_listens: u32,
    #[serde(default)]
    pub total_user_listens: u32,
    #[serde(default)]
    pub total_likes: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlbumWithArtists {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub title: String,
    pub cover_url: Option<String>,
    pub release_year: Option<u16>,
    pub genres: Vec<String>,
    pub langs: Vec<String>,
    pub dominant_color: Option<String>,
    pub total_tracks: u32,
    pub total_duration: Duration,
    #[serde(default)]
    pub total_listens: u32,
    #[serde(default)]
    pub total_user_listens: u32,
    #[serde(default)]
    pub total_likes: u32,
    pub artists: Vec<Artist>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlbumWithRelations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub title: String,
    pub cover_url: Option<String>,
    pub release_year: Option<u16>,
    pub genres: Vec<String>,
    pub langs: Vec<String>,
    pub dominant_color: Option<String>,
    pub total_tracks: u32,
    pub total_duration: Duration,
    #[serde(default)]
    pub total_listens: u32,
    #[serde(default)]
    pub total_user_listens: u32,
    #[serde(default)]
    pub total_likes: u32,
    pub artists: Vec<Artist>,
    pub songs: Vec<Song>,
}
