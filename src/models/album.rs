use serde::{Deserialize, Serialize};
use surrealdb::{
    sql::{Duration, Thing},
    Datetime,
};

use crate::models::{artist::Artist, song::Song};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Album {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,

    pub title: String,
    pub cover_url: String,
    pub release_date: Option<Datetime>,
    pub genres: Vec<String>,
    pub langs: Vec<String>,
    pub dominant_color: String,
    pub total_tracks: u32,
    pub total_duration: Duration,

    // Stats
    pub total_listens: u32,
    pub total_user_listen: u32,
    pub total_likes: u32,

    // Relations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artists: Option<Vec<Artist>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub songs: Option<Vec<Song>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlbumWithRelations {
    pub id: Thing,
    pub title: String,
    pub artists: Vec<Artist>,
    pub cover_url: String,
    pub langs: Vec<String>,
    pub genres: Vec<String>,
    pub dominant_color: String,
    #[serde(default)]
    pub songs: Vec<Song>,
}
