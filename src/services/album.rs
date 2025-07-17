use surrealdb::{engine::any::Any, sql::Thing, Error, Surreal};

use crate::models::album::{AlbumWithArtists, AlbumWithRelations};

pub async fn get_album_service(
    db: &Surreal<Any>,
    album_id: Thing,
) -> Result<Option<AlbumWithRelations>, Error> {
    let sql_query = "
        SELECT *,
        (SELECT * FROM (SELECT VALUE in FROM <-artist_creates_album)) AS artists,
        (SELECT * FROM (SELECT VALUE out FROM ->album_contains_song) ORDER BY song_index ASC) AS songs
        FROM $album_id;
    ";

    let mut response = db.query(sql_query).bind(("album_id", album_id)).await?;

    let album: Option<AlbumWithRelations> = response.take(0)?;

    Ok(album)
}

pub async fn get_albums_service(db: &Surreal<Any>) -> Result<Vec<AlbumWithArtists>, Error> {
    let sql_query = "SELECT *, <-artist_creates_album<-artist.* AS artists FROM album;";

    let mut response = db.query(sql_query).await?;

    let albums: Vec<AlbumWithArtists> = response.take(0)?;

    Ok(albums)
}
