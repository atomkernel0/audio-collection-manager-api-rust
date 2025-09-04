use surrealdb::sql::Thing;

#[derive(serde::Deserialize)]
pub struct CountResult {
    pub total: u64,
}

#[derive(serde::Deserialize)]
pub struct RelationId {
    pub id: Thing,
}