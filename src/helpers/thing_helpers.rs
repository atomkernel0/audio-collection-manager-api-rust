use surrealdb::sql::Thing;

/// Utility functions for creating SurrealDB `Thing` records.
///
/// A `Thing` is a unique record identifier in SurrealDB, composed of a table name
/// and a specific record ID (e.g., `user:123`). These helpers simplify the
/// process of creating `Thing`s from ID strings.

// ---

pub fn thing_to_string(thing: &Thing) -> String {
    let tb = &thing.tb;
    let id_str = match &thing.id {
        surrealdb::sql::Id::Number(n) => n.to_string(),
        surrealdb::sql::Id::String(s) => s.to_owned(),
        surrealdb::sql::Id::Array(_) => thing.id.to_raw(),
        surrealdb::sql::Id::Generate(_) => thing.id.to_raw(),
        id => id.to_raw(),
    };

    // Nettoie les éventuels ⟨⟩ ou guillemets inutiles
    let clean_id = id_str
        .trim_matches(|c: char| c == '⟨' || c == '⟩' || c == '"' || c == '\'')
        .to_string();

    format!("{}:{}", tb, clean_id)
}

/// Parses a record ID string to extract the ID part.
///
/// This function handles two common formats:
/// - A full record link like `"table:id"`, from which it extracts `"id"`.
/// - A simple ID like `"id"`, which it returns as is.
///
/// # Arguments
///
/// * `id` - The ID string to parse, which can be in the format `"table:id"` or `"id"`.
///
/// # Examples
///
/// ```
/// // Assuming this function is in a module accessible from the crate root
/// // use my_crate::utils::parse_id_part;
///
/// # fn parse_id_part(id: &str) -> &str {
/// #     if let Some(id_part) = id.split(':').nth(1) { id_part } else { id }
/// # }
///
/// assert_eq!(parse_id_part("user:123"), "123");
/// assert_eq!(parse_id_part("456"), "456");
/// ```
pub fn parse_id_part(id: &str) -> &str {
    // If the ID contains a table prefix like "table:id", extract just the ID part.
    if let Some(id_part) = id.split(':').nth(1) {
        id_part
    } else {
        // Otherwise, the string is already the ID.
        id
    }
}

/// Creates a `Thing` for an album record.
pub fn create_album_thing(album_id: &str) -> Thing {
    let clean_id = parse_id_part(album_id);
    Thing::from(("album".to_string(), clean_id.to_string()))
}

/// Creates a `Thing` for a user record.
pub fn create_user_thing(user_id: &str) -> Thing {
    let clean_id = parse_id_part(user_id);
    Thing::from(("user".to_string(), clean_id.to_string()))
}

/// Creates a `Thing` for a song record.
pub fn create_song_thing(song_id: &str) -> Thing {
    let clean_id = parse_id_part(song_id);
    Thing::from(("song".to_string(), clean_id.to_string()))
}

/// Creates a `Thing` for an artist record.
pub fn create_artist_thing(artist_id: &str) -> Thing {
    let clean_id = parse_id_part(artist_id);
    Thing::from(("artist".to_string(), clean_id.to_string()))
}

/// Creates a `Thing` for a playlist record.
pub fn create_playlist_thing(playlist_id: &str) -> Thing {
    let clean_id = parse_id_part(playlist_id);
    Thing::from(("playlist".to_string(), clean_id.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_user_id() {
        assert_eq!(parse_id_part("user:123"), "123");
        assert_eq!(parse_id_part("123"), "123");
        assert_eq!(parse_id_part("user:test_user"), "test_user");
        assert_eq!(parse_id_part("playlist:test_playlist"), "test_playlist");
    }

    #[tokio::test]
    async fn test_create_things() {
        let user_thing = create_user_thing("user:12");
        assert_eq!(user_thing.tb, "user");
        assert_eq!(user_thing.id.to_string(), "⟨12⟩");

        let album_thing: Thing = create_album_thing("album:34");
        assert_eq!(album_thing.tb, "album");
        assert_eq!(album_thing.id.to_string(), "⟨34⟩");

        let song_thing = create_song_thing("song:56");
        assert_eq!(song_thing.tb, "song");
        assert_eq!(song_thing.id.to_string(), "⟨56⟩");

        let artist_thing = create_artist_thing("artist:78");
        assert_eq!(artist_thing.tb, "artist");
        assert_eq!(artist_thing.id.to_string(), "⟨78⟩");

        let playlist_thing = create_playlist_thing("playlist:90");
        assert_eq!(playlist_thing.tb, "playlist");
        assert_eq!(playlist_thing.id.to_string(), "⟨90⟩");
    }
}
