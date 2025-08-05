use crate::{
    helpers::thing_helpers::create_album_thing,
    models::album::{AlbumWithArtists, AlbumWithRelations},
};
use surrealdb::{engine::any::Any, Error, Surreal};

pub struct AlbumService;

impl AlbumService {
    pub async fn get_albums(db: &Surreal<Any>) -> Result<Vec<AlbumWithArtists>, Error> {
        let sql_query = "SELECT *, <-artist_creates_album<-artist.* AS artists FROM album;";

        let mut response = db.query(sql_query).await?;

        let albums: Vec<AlbumWithArtists> = response.take(0)?;

        Ok(albums)
    }

    pub async fn get_album(
        db: &Surreal<Any>,
        album_id: &str,
    ) -> Result<Option<AlbumWithRelations>, Error> {
        let album_thing = create_album_thing(album_id);

        let sql_query = "
        SELECT *,
        (SELECT * FROM (SELECT VALUE in FROM <-artist_creates_album)) AS artists,
        (SELECT * FROM (SELECT VALUE out FROM ->album_contains_song) ORDER BY song_index ASC) AS songs
        FROM $album_thing;
    ";

        let mut response = db
            .query(sql_query)
            .bind(("album_thing", album_thing))
            .await?;

        let album: Option<AlbumWithRelations> = response.take(0)?;

        Ok(album)
    }
}
