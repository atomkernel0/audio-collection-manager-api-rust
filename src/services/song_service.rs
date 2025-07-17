use crate::error::Result;
use crate::helpers::song_helpers::song_exists;
use crate::models::album::AlbumWithRelations;
use crate::Error;
use surrealdb::{engine::any::Any, Surreal};

use surrealdb::sql::Thing;

pub async fn listen_to_song(
    db: &Surreal<Any>,
    song_id: &str,
    user_id: Option<&str>,
) -> Result<bool> {
    let song_thing = Thing::from(("song", song_id));

    if !song_exists(db, &song_id).await? {
        return Err(Error::SongNotFound {
            id: song_id.to_string(),
        });
    }

    let user_id_owned = user_id.map(String::from);

    let sql_query = "UPDATE $song_id SET total_listens += 1, total_user_listens = if $user_id THEN total_user_listens + 1 ELSE total_user_listens END;";

    db.query(sql_query)
        .bind(("song_id", song_thing))
        .bind(("user_id", user_id_owned))
        .await?;

    Ok(true)
}

pub async fn get_album_from_song(
    db: &Surreal<Any>,
    song_id: &str,
) -> Result<Option<AlbumWithRelations>> {
    let song_thing = Thing::from(("song", song_id));

    let sql_query = "SELECT *,
        (SELECT * FROM (SELECT VALUE in FROM <-artist_creates_album)) AS artists,
        (SELECT * FROM (SELECT VALUE out FROM ->album_contains_song) ORDER BY song_index) AS songs
        FROM (SELECT VALUE array::first(<-album_contains_song<-album[*]) 
        FROM $song_id);";

    let mut response = db.query(sql_query).bind(("song_id", song_thing)).await?;

    let album: Option<AlbumWithRelations> = response.take(0)?;

    Ok(album)
}
