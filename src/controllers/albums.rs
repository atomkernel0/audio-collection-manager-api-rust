use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::{
    models::albums::{Album, GetAllAlbumsResponse},
    Result,
};

pub async fn get_all_albums(db: &Surreal<Client>) -> Result<Vec<GetAllAlbumsResponse>> {
    let mut result = db
        .query("SELECT *, array::len(songs) AS song_length FROM album")
        .await?;

    let db_albums: Vec<GetAllAlbumsResponse> = result.take(0)?;

    Ok(db_albums
        .into_iter()
        .map(GetAllAlbumsResponse::from)
        .collect())
}

// pub async fn get_album_by_id(db: &Surreal<Client>, id: &str) -> Result<Option<Album>> {
//     let mut result = db
//         .query("SELECT * FROM album WHERE id = $id")
//         .bind(("id", id))
//         .await?;

//     let album: Option<Album> = result.take(0)?;
//     Ok(album)
// }
