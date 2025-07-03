use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use crate::models::{album::Album, song::Song};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artist {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,

    pub name: String,
    pub genres: Vec<String>,
    pub country: Option<String>,

    //pub monthly_listeners: u32, //TODO: implement ?

    // Relations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub albums: Option<Vec<Album>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub songs: Option<Vec<Song>>,
}
