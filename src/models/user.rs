use serde::{Deserialize, Serialize};
use surrealdb::{sql::Thing, Datetime};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub username: String,
    pub password: String,
    pub created_at: Datetime,
    pub listen_count: u32,
    pub total_listening_time: u64,
    pub favorite_count: u16,

    pub listening_streak: u16,
    pub badges: Vec<Badge>,

    //NOTE: idk if I will implement the following
    pub level: u16,
    pub experience_points: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub username: String,
    // no password
    pub created_at: Datetime,
    pub listen_count: u32,
    pub total_listening_time: u64,
    pub favorite_count: u16,

    pub listening_streak: u16,
    pub badges: Vec<Badge>,

    //NOTE: idk if I will implement the following
    pub level: u16,
    pub experience_points: u32,
}

#[allow(dead_code)] //TODO: remove this
#[derive(strum_macros::Display, Debug, Serialize, Deserialize, Clone)]
pub enum Badge {
    // Listening time achievements
    Listen10Hours,   // Bronze badge: 10 hours of total listening
    Listen50Hours,   // Silver badge: 50 hours of total listening
    Listen100Hours,  // Gold badge: 100 hours of total listening
    Listen500Hours,  // Platinum badge: 500 hours of total listening
    Listen1000Hours, // Diamond badge: 1000 hours of total listening

    // Favorite tracking achievements
    Favorite10Song,  // Bronze badge: 10 songs added to favorites
    Favorite20Song,  // Silver badge: 20 songs added to favorites
    Favorite50Song,  // Gold badge: 50 songs added to favorites
    Favorite100Song, // Platinum badge: 100 songs added to favorites
    Favorite200Song, // Diamond badge: 200 songs added to favorites

    // Playlist contributions achievements
    Playlist10Song,  // Bronze badge: 10 songs added to playlists
    Playlist30Song,  // Silver badge: 30 songs added to playlists
    Playlist70Song,  // Gold badge: 70 songs added to playlists
    Playlist150Song, // Platinum badge: 150 songs added to playlists
    Playlist250Song, // Diamond badge: 250 songs added to playlists
}
