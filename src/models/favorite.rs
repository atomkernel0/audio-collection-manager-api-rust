use serde::{Deserialize, Serialize};
use surrealdb::{sql::Thing, Datetime};

/// Métadonnées étendues pour les relations de favoris
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FavoriteMetadata {
    /// Tags personnalisés pour catégoriser les favoris
    pub tags: Vec<String>,

    /// Notes personnelles de l'utilisateur
    pub notes: Option<String>,

    /// Note personnelle (1-5 étoiles)
    pub user_rating: Option<u8>,

    /// Ordre de tri personnalisé dans les listes de favoris
    pub sort_order: i32,

    /// Distinction entre "like" simple et "favori" marqué
    pub is_favorite: bool,

    /// Timestamp du dernier accès pour les recommandations
    pub last_accessed: Option<Datetime>,

    /// Timestamp de création de la relation
    pub created_at: Datetime,
}

impl FavoriteMetadata {
    /// Valide les tags (longueur, caractères autorisés)
    pub fn validate_tags(&self) -> Result<(), String> {
        for tag in &self.tags {
            if tag.is_empty() || tag.len() > 50 {
                return Err("Les tags doivent faire entre 1 et 50 caractères".to_string());
            }
            if !tag
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                return Err(
                    "Les tags ne peuvent contenir que des lettres, chiffres, _ et -".to_string(),
                );
            }
        }
        Ok(())
    }

    /// Valide les notes (longueur maximale)
    pub fn validate_notes(&self) -> Result<(), String> {
        if let Some(notes) = &self.notes {
            if notes.len() > 1000 {
                return Err("Les notes ne peuvent pas dépasser 1000 caractères".to_string());
            }
        }
        Ok(())
    }

    /// Validation complète
    pub fn validate(&self) -> Result<(), String> {
        self.validate_tags()?;
        self.validate_notes()?;
        Ok(())
    }
}

/// Préférences utilisateur pour la gestion des favoris
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserFavoritesPreferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,

    /// Référence vers l'utilisateur
    pub user_id: Thing,

    /// Tri par défaut ('created_at', 'user_rating', 'sort_order', 'last_accessed', 'title')
    pub default_sort_by: String,

    /// Ordre de tri par défaut ('ASC', 'DESC')
    pub default_sort_order: String,

    /// Nombre d'éléments par page
    pub items_per_page: u32,

    /// Afficher les ratings dans l'interface
    pub show_ratings: bool,

    /// Afficher les notes dans l'interface
    pub show_notes: bool,

    /// Afficher les tags dans l'interface
    pub show_tags: bool,

    /// Ajouter automatiquement aux favoris lors du like
    pub auto_add_to_favorites: bool,

    /// Format d'export par défaut ('json', 'csv', 'xml')
    pub export_format: String,

    /// Timestamp de création
    pub created_at: Datetime,

    /// Timestamp de dernière mise à jour
    pub updated_at: Datetime,
}

impl UserFavoritesPreferences {
    /// Valide les préférences utilisateur
    pub fn validate(&self) -> Result<(), String> {
        // Validation du critère de tri
        let valid_sort_by = [
            "created_at",
            "user_rating",
            "sort_order",
            "last_accessed",
            "title",
        ];
        if !valid_sort_by.contains(&self.default_sort_by.as_str()) {
            return Err("Critère de tri invalide".to_string());
        }

        // Validation de l'ordre de tri
        if !["ASC", "DESC"].contains(&self.default_sort_order.as_str()) {
            return Err("Ordre de tri invalide".to_string());
        }

        // Validation du nombre d'éléments par page
        if self.items_per_page < 1 || self.items_per_page > 100 {
            return Err("Le nombre d'éléments par page doit être entre 1 et 100".to_string());
        }

        // Validation du format d'export
        if !["json", "csv", "xml"].contains(&self.export_format.as_str()) {
            return Err("Format d'export invalide".to_string());
        }

        Ok(())
    }
}

/// Options de tri pour les requêtes de favoris
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FavoriteSortBy {
    CreatedAt,
    UserRating,
    SortOrder,
    LastAccessed,
    Title,
}

impl Default for FavoriteSortBy {
    fn default() -> Self {
        FavoriteSortBy::CreatedAt
    }
}

impl FavoriteSortBy {
    pub fn to_string(&self) -> String {
        match self {
            FavoriteSortBy::CreatedAt => "created_at".to_string(),
            FavoriteSortBy::UserRating => "user_rating".to_string(),
            FavoriteSortBy::SortOrder => "sort_order".to_string(),
            FavoriteSortBy::LastAccessed => "last_accessed".to_string(),
            FavoriteSortBy::Title => "title".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Result<Self, String> {
        match s {
            "created_at" => Ok(FavoriteSortBy::CreatedAt),
            "user_rating" => Ok(FavoriteSortBy::UserRating),
            "sort_order" => Ok(FavoriteSortBy::SortOrder),
            "last_accessed" => Ok(FavoriteSortBy::LastAccessed),
            "title" => Ok(FavoriteSortBy::Title),
            _ => Err(format!("Critère de tri invalide: {}", s)),
        }
    }
}

/// Ordre de tri
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SortOrder {
    ASC,
    DESC,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::DESC
    }
}

impl SortOrder {
    pub fn to_string(&self) -> String {
        match self {
            SortOrder::ASC => "ASC".to_string(),
            SortOrder::DESC => "DESC".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "ASC" => Ok(SortOrder::ASC),
            "DESC" => Ok(SortOrder::DESC),
            _ => Err(format!("Ordre de tri invalide: {}", s)),
        }
    }
}

/// Format d'export des favoris
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    Xml,
}

impl Default for ExportFormat {
    fn default() -> Self {
        ExportFormat::Json
    }
}

impl ExportFormat {
    pub fn to_string(&self) -> String {
        match self {
            ExportFormat::Json => "json".to_string(),
            ExportFormat::Csv => "csv".to_string(),
            ExportFormat::Xml => "xml".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "csv" => Ok(ExportFormat::Csv),
            "xml" => Ok(ExportFormat::Xml),
            _ => Err(format!("Format d'export invalide: {}", s)),
        }
    }
}

/// Relation utilisateur-album avec métadonnées étendues
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserLikesAlbum {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,

    /// Utilisateur (in)
    #[serde(rename = "in")]
    pub user_id: Thing,

    /// Album (out)
    #[serde(rename = "out")]
    pub album_id: Thing,

    /// Métadonnées étendues
    #[serde(flatten)]
    pub metadata: FavoriteMetadata,
}

/// Relation utilisateur-chanson avec métadonnées étendues
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserLikesSong {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,

    /// Utilisateur (in)
    #[serde(rename = "in")]
    pub user_id: Thing,

    /// Chanson (out)
    #[serde(rename = "out")]
    pub song_id: Thing,

    /// Métadonnées étendues
    #[serde(flatten)]
    pub metadata: FavoriteMetadata,
}

/// Relation utilisateur-artiste avec métadonnées étendues
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserLikesArtist {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,

    /// Utilisateur (in)
    #[serde(rename = "in")]
    pub user_id: Thing,

    /// Artiste (out)
    #[serde(rename = "out")]
    pub artist_id: Thing,

    /// Métadonnées étendues
    #[serde(flatten)]
    pub metadata: FavoriteMetadata,
}

/// Réponse paginée pour les favoris
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FavoritesResponse<T> {
    /// Éléments de la page courante
    pub data: Vec<T>,

    /// Informations de pagination
    pub pagination: PaginationInfo,
}

/// Informations de pagination
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginationInfo {
    /// Numéro de la page courante (1-based)
    pub current_page: u32,

    /// Nombre total de pages
    pub total_pages: u32,

    /// Nombre total d'éléments
    pub total_items: u64,

    /// Nombre d'éléments par page
    pub page_size: u32,

    /// Y a-t-il une page suivante
    pub has_next_page: bool,

    /// Y a-t-il une page précédente
    pub has_previous_page: bool,
}

/// Album avec métadonnées de favori
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlbumWithFavoriteMetadata {
    /// Données de l'album
    #[serde(flatten)]
    pub album: crate::models::album::AlbumWithArtists,

    /// Métadonnées de favori (si l'utilisateur a liké)
    pub favorite_metadata: Option<FavoriteMetadata>,
}

/// Chanson avec métadonnées de favori
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SongWithFavoriteMetadata {
    /// Données de la chanson
    #[serde(flatten)]
    pub song: crate::models::song::Song,

    /// Métadonnées de favori (si l'utilisateur a liké)
    pub favorite_metadata: Option<FavoriteMetadata>,
}

/// Artiste avec métadonnées de favori
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtistWithFavoriteMetadata {
    /// Données de l'artiste
    #[serde(flatten)]
    pub artist: crate::models::artist::Artist,

    /// Métadonnées de favori (si l'utilisateur a liké)
    pub favorite_metadata: Option<FavoriteMetadata>,
}

/// Paramètres de requête pour les favoris
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FavoritesQuery {
    /// Numéro de page (1-based)
    pub page: Option<u32>,

    /// Nombre d'éléments par page
    pub page_size: Option<u32>,

    /// Critère de tri
    pub sort_by: Option<String>,

    /// Ordre de tri
    pub sort_direction: Option<String>,

    /// Filtrer par favoris marqués uniquement
    pub favorites_only: Option<bool>,

    /// Filtrer par tags (intersection)
    pub tags: Option<Vec<String>>,

    /// Filtrer par rating minimum
    pub min_rating: Option<u8>,

    /// Filtrer par rating maximum
    pub max_rating: Option<u8>,
}

impl Default for FavoritesQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
            sort_by: Some("created_at".to_string()),
            sort_direction: Some("DESC".to_string()),
            favorites_only: None,
            tags: None,
            min_rating: None,
            max_rating: None,
        }
    }
}

/// Données pour mettre à jour les métadonnées d'un favori
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateFavoriteRequest {
    /// Tags à ajouter/modifier
    pub tags: Option<Vec<String>>,

    /// Note à ajouter/modifier
    pub notes: Option<String>,

    /// Rating à ajouter/modifier (1-5)
    pub user_rating: Option<u8>,

    /// Nouvel ordre de tri
    pub sort_order: Option<i32>,

    /// Marquer/démarquer comme favori
    pub is_favorite: Option<bool>,
}

/// Données pour ajouter un favori
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddFavoriteRequest {
    /// ID de l'élément à ajouter aux favoris
    pub item_id: Thing,

    /// Type d'élément ('album', 'song', 'artist')
    pub item_type: String,

    /// Tags initiaux
    pub tags: Option<Vec<String>>,

    /// Note initiale
    pub notes: Option<String>,

    /// Rating initial (1-5)
    pub user_rating: Option<u8>,

    /// Ordre de tri initial
    pub sort_order: Option<i32>,

    /// Marquer comme favori dès l'ajout
    pub is_favorite: Option<bool>,
}

/// Opération en lot sur les favoris
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BulkFavoriteOperation {
    /// Type d'opération ('add', 'remove', 'update')
    pub operation: String,

    /// Type d'élément ('album', 'song', 'artist')
    pub item_type: String,

    /// IDs des éléments concernés
    pub item_ids: Vec<Thing>,

    /// Données de mise à jour (pour l'opération 'update')
    pub update_data: Option<UpdateFavoriteRequest>,
}

/// Données d'export des favoris
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FavoritesExport {
    /// Timestamp de l'export
    pub exported_at: Datetime,

    /// Utilisateur qui a exporté
    pub user_id: Thing,

    /// Albums favoris
    pub albums: Vec<AlbumWithFavoriteMetadata>,

    /// Chansons favorites
    pub songs: Vec<SongWithFavoriteMetadata>,

    /// Artistes favoris
    pub artists: Vec<ArtistWithFavoriteMetadata>,

    /// Préférences utilisateur
    pub preferences: UserFavoritesPreferences,
}

/// Statistiques des favoris
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FavoritesStatistics {
    /// Nombre total d'albums favoris
    pub total_albums: u64,

    /// Nombre total de chansons favorites
    pub total_songs: u64,

    /// Nombre total d'artistes favoris
    pub total_artists: u64,

    /// Temps total d'écoute des favoris (en secondes)
    pub total_play_time: u64,

    /// Genres les plus écoutés
    pub most_played_genres: Vec<GenreCount>,

    /// Éléments récemment ajoutés
    pub recently_added: RecentlyAddedFavorites,
}

/// Compteur par genre
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenreCount {
    pub genre: String,
    pub count: u64,
}

/// Favoris récemment ajoutés
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecentlyAddedFavorites {
    pub albums: Vec<AlbumWithFavoriteMetadata>,
    pub songs: Vec<SongWithFavoriteMetadata>,
    pub artists: Vec<ArtistWithFavoriteMetadata>,
}

/// Résultat de recherche dans les favoris
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FavoritesSearchResult {
    pub albums: Vec<AlbumWithFavoriteMetadata>,
    pub songs: Vec<SongWithFavoriteMetadata>,
    pub artists: Vec<ArtistWithFavoriteMetadata>,
}

/// Résultat d'import de favoris
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportResult {
    pub imported: u64,
    pub failed: u64,
    pub errors: Vec<String>,
}

/// Résultat de synchronisation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncResult {
    pub synced: u64,
    pub conflicts: u64,
}
