use surrealdb::{engine::any::Any, sql::Thing, Error, Surreal};

use crate::models::album::{Album, AlbumWithRelations};

pub async fn get_album_service(
    db: &Surreal<Any>,
    album_id: Thing,
) -> Result<Option<AlbumWithRelations>, Error> {
    let sql_query = "SELECT *, ->album_contains_song->song.* AS songs FROM type::thing($tb, $id);";

    let mut response = db
        .query(sql_query)
        .bind(("tb", "album"))
        .bind(("id", album_id))
        .await?;

    let album: Option<AlbumWithRelations> = response.take(0)?;

    Ok(album)
}

pub async fn get_albums_service(db: &Surreal<Any>) -> Result<Vec<Album>, Error> {
    let sql_query = "SELECT * FROM album";

    let mut response = db.query(sql_query).await?;

    let albums: Vec<Album> = response.take(0)?;

    Ok(albums)
}
