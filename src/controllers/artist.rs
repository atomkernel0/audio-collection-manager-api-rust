use surrealdb::{engine::any::Any, sql::Thing, Error, Surreal};

use crate::{
    models::artist::{Artist, ArtistWithAlbumsAndTopSongs},
    services,
};

pub async fn get_artists(db: &Surreal<Any>) -> Result<Vec<Artist>, Error> {
    services::artist::get_artists_service(db).await
}

pub async fn get_artist(
    db: &Surreal<Any>,
    artist_id: Thing,
) -> Result<Option<ArtistWithAlbumsAndTopSongs>, Error> {
    services::artist::get_artist_service(db, artist_id).await
}
