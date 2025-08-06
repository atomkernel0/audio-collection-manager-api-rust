use serde::{Deserialize, Serialize};
use surrealdb::Datetime;

use crate::models::{
    album::AlbumWithArtists, artist::ArtistWithAlbums, pagination::PaginationInfo,
    song::SongWithRelations,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FavoritesResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumWithFavoriteMetadata {
    pub album: AlbumWithArtists,
    #[serde(default)]
    pub sort_order: i32,
    pub last_accessed: Option<Datetime>,
    pub favorited_at: Datetime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SongWithFavoriteMetadata {
    pub song: SongWithRelations,
    #[serde(default)]
    pub sort_order: i32,
    pub last_accessed: Option<Datetime>,
    pub favorited_at: Datetime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtistWithFavoriteMetadata {
    pub artist: ArtistWithAlbums,
    #[serde(default)]
    pub sort_order: i32,
    pub last_accessed: Option<Datetime>,
    pub favorited_at: Datetime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FavoritesQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
}

impl Default for FavoritesQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
            sort_by: Some("created_at".to_string()),
            sort_direction: Some("DESC".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FavoritesStatistics {
    pub total_albums: u64,
    pub total_songs: u64,
    pub total_artists: u64,
    pub total_play_time: u64,
    pub most_played_genres: Vec<GenreCount>,
    pub recently_added: RecentlyAddedFavorites,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenreCount {
    pub genre: String,
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentlyAddedFavorites {
    pub albums: Vec<AlbumWithFavoriteMetadata>,
    pub songs: Vec<SongWithFavoriteMetadata>,
    pub artists: Vec<ArtistWithFavoriteMetadata>,
}
