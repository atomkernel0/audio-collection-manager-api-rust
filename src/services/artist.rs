use surrealdb::{engine::any::Any, sql::Thing, Error, Surreal};

use crate::models::artist::{Artist, ArtistWithAlbumsAndTopSongs};

pub async fn get_artists_service(db: &Surreal<Any>) -> Result<Vec<Artist>, Error> {
    let sql_query = "SELECT * FROM artist;";

    let mut response = db.query(sql_query).await?;

    let artists: Vec<Artist> = response.take(0)?;

    Ok(artists)
}

pub async fn get_artist_service(
    db: &Surreal<Any>,
    artist_id: Thing,
) -> Result<Option<ArtistWithAlbumsAndTopSongs>, Error> {
    /*  let sql_query = "
        SELECT *,
        (SELECT * FROM (SELECT VALUE out FROM ->artist_creates_album)) AS albums,
        (SELECT * FROM (SELECT VALUE out FROM ->artist_performs_song )) AS top_songs
        FROM $artist_id;";
    */

    let sql_query = "SELECT *,
        (SELECT *, <-artist_creates_album<-artist[*] AS artists FROM (SELECT VALUE out FROM ->artist_creates_album)) AS albums,
        (SELECT * FROM (SELECT VALUE out FROM ->artist_performs_song)) AS top_songs
        FROM $artist_id;";

    let mut response = db.query(sql_query).bind(("artist_id", artist_id)).await?;

    let artist: Option<ArtistWithAlbumsAndTopSongs> = response.take(0)?;

    Ok(artist)
}
