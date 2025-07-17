use serde::{Deserialize, Serialize};
use surrealdb::{sql::Duration, sql::Thing, Datetime};

use crate::models::{song::Song, user::User};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Playlist {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,

    pub name: String,
    pub cover_url: Option<String>,
    pub is_public: bool,
    pub dominant_color: Option<String>,
    pub created_by: Thing,

    // Timestamps
    pub created_at: Datetime,
    pub updated_at: Datetime,

    // Stats
    pub songs_count: u32,
    pub total_duration: Duration,
    pub total_listens: u32,
    pub total_likes: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistWithSongs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub name: String,
    pub cover_url: Option<String>,
    pub is_public: bool,
    pub dominant_color: Option<String>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub songs_count: u32,
    pub total_duration: Duration,
    pub total_listens: u32,
    pub total_likes: u32,
    pub created_by: User,
    pub songs: Vec<Song>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistSong {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,

    #[serde(rename = "in")]
    pub playlist_id: Thing,

    #[serde(rename = "out")]
    pub song_id: Thing,

    pub position: u32,
    pub added_at: Datetime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub cover_url: Option<String>,
    pub is_public: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdatePlaylistRequest {
    pub name: Option<String>,
    pub cover_url: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub is_public: Option<bool>,
    pub user_id: Option<Thing>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistResponse {
    pub data: Vec<PlaylistWithSongs>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginationInfo {
    pub current_page: u32,
    pub total_pages: u32,
    pub total_items: u64,
    pub page_size: u32,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

impl Default for PlaylistQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
            is_public: None,
            user_id: None,
            search: None,
            sort_by: Some("created_at".to_string()),
            sort_order: Some("DESC".to_string()),
        }
    }
}
