use surrealdb::{engine::any::Any, sql::Thing, Error, Surreal};

use crate::{
    models::album::{Album, AlbumWithRelations},
    services,
};

pub async fn get_album(
    db: &Surreal<Any>,
    album_id: Thing,
) -> Result<Option<AlbumWithRelations>, Error> {
    services::album::get_album_service(db, album_id).await
}

pub async fn get_albums(db: &Surreal<Any>) -> Result<Vec<Album>, Error> {
    services::album::get_albums_service(db).await
}
