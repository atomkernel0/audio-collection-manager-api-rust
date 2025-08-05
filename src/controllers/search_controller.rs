use crate::error::Result;
use crate::services::search_service::SearchService;
use crate::{services::search_service::SearchResult, AppState};
use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub term: String,
}

pub struct SearchController;

impl SearchController {
    pub async fn search_albums_songs_artists(
        State(state): State<AppState>,
        Query(params): Query<SearchQuery>,
    ) -> Result<Json<SearchResult>> {
        let term = params.term.trim();

        if term.is_empty() {
            return Err(crate::error::Error::InvalidInput {
                reason: "Search term cannot be empty".to_string(),
            });
        }

        if term.len() > 50 {
            return Err(crate::error::Error::InvalidInput {
                reason: "Search term cannot be longer than 50 characters".to_string(),
            });
        }

        let result: SearchResult =
            SearchService::search_albums_songs_artists(&state.db, term).await?;

        Ok(Json(result))
    }
}
