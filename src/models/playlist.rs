use surrealdb::sql::Thing;

pub struct Playlist {
    pub id: Thing,
    pub name: String,
    pub is_public: bool,
    // created_by: User
}
