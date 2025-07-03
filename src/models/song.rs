use serde::{Deserialize, Serialize};
use surrealdb::sql::{Duration, Thing};

use crate::models::{album::Album, artist::Artist};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Song {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,

    pub title: String,
    pub file_url: String,
    pub duration: Duration,

    // Audio metadata for recommendations
    pub tempo: f32, // BPM

    // Stats
    pub total_listens: u32,
    pub total_user_listens: u32,
    pub total_likes: u32,

    // Relations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artists: Option<Vec<Artist>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<Album>,
}
