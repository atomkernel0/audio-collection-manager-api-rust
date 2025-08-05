use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

/// Informations de pagination
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginationInfo {
    pub current_page: u32, // (1-based)
    pub total_pages: u32,
    pub total_items: u64,
    pub page_size: u32, // number of elements per page
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginationQuery {
    pub page: Option<u32>, // 1-based
    pub page_size: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
            sort_by: Some("created_at".to_string()),
            sort_direction: Some("DESC".to_string()),
        }
    }
}
