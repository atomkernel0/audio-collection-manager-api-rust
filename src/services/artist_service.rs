use surrealdb::{engine::any::Any, Error, Surreal};

use crate::{
    helpers::thing_helpers::create_artist_thing,
    models::artist::{Artist, ArtistWithAlbumsAndTopSongs},
};

pub struct ArtistService;

impl ArtistService {
    pub async fn get_artists(db: &Surreal<Any>) -> Result<Vec<Artist>, Error> {
        let sql_query = "SELECT * FROM artist;";

        let mut response = db.query(sql_query).await?;

        let artists: Vec<Artist> = response.take(0)?;

        Ok(artists)
    }

    pub async fn get_artist(
        db: &Surreal<Any>,
        artist_id: &str,
    ) -> Result<Option<ArtistWithAlbumsAndTopSongs>, Error> {
        let artist_thing = create_artist_thing(artist_id);

        let sql_query = "SELECT *,
        (SELECT *, <-artist_creates_album<-artist[*] AS artists FROM (SELECT VALUE out FROM ->artist_creates_album)) AS albums,
        (SELECT * FROM (SELECT VALUE out FROM ->artist_performs_song)) AS top_songs
        FROM $artist_id;";

        let mut response = db
            .query(sql_query)
            .bind(("artist_id", artist_thing))
            .await?;

        let artist: Option<ArtistWithAlbumsAndTopSongs> = response.take(0)?;

        Ok(artist)
    }
}
