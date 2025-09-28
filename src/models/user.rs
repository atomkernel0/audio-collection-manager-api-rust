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
    pub badges: Vec<BadgeEnum>,

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
    pub badges: Vec<BadgeEnum>,

    //NOTE: idk if I will implement the following
    pub level: u16,
    pub experience_points: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum BadgeEnum {
    // Listening time achievements
    #[serde(rename = "listen_10_hours")]
    Listen10Hours,
    #[serde(rename = "listen_50_hours")]
    Listen50Hours,
    #[serde(rename = "listen_100_hours")]
    Listen100Hours,
    #[serde(rename = "listen_500_hours")]
    Listen500Hours,
    #[serde(rename = "listen_1000_hours")]
    Listen1000Hours,

    // Favorite tracking achievements
    #[serde(rename = "favorite_10_song")]
    Favorite10Song,
    #[serde(rename = "favorite_20_song")]
    Favorite20Song,
    #[serde(rename = "favorite_50_song")]
    Favorite50Song,
    #[serde(rename = "favorite_100_song")]
    Favorite100Song,
    #[serde(rename = "favorite_200_song")]
    Favorite200Song,

    // Playlist contributions achievements
    #[serde(rename = "playlist_10_song")]
    Playlist10Song,
    #[serde(rename = "playlist_30_song")]
    Playlist30Song,
    #[serde(rename = "playlist_70_song")]
    Playlist70Song,
    #[serde(rename = "playlist_150_song")]
    Playlist150Song,
    #[serde(rename = "playlist_250_song")]
    Playlist250Song,
}

impl std::fmt::Display for BadgeEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BadgeEnum::Listen10Hours => "listen_10_hours",
            BadgeEnum::Listen50Hours => "listen_50_hours",
            BadgeEnum::Listen100Hours => "listen_100_hours",
            BadgeEnum::Listen500Hours => "listen_500_hours",
            BadgeEnum::Listen1000Hours => "listen_1000_hours",
            BadgeEnum::Favorite10Song => "favorite_10_song",
            BadgeEnum::Favorite20Song => "favorite_20_song",
            BadgeEnum::Favorite50Song => "favorite_50_song",
            BadgeEnum::Favorite100Song => "favorite_100_song",
            BadgeEnum::Favorite200Song => "favorite_200_song",
            BadgeEnum::Playlist10Song => "playlist_10_song",
            BadgeEnum::Playlist30Song => "playlist_30_song",
            BadgeEnum::Playlist70Song => "playlist_70_song",
            BadgeEnum::Playlist150Song => "playlist_150_song",
            BadgeEnum::Playlist250Song => "playlist_250_song",
        };
        write!(f, "{}", s)
    }
}
