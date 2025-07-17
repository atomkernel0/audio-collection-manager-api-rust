use surrealdb::sql::Thing;

/// Parse un ID depuis différents formats possibles
pub fn parse_id_part(id: &str) -> &str {
    // Si l'ID contient déjà "tb:id", on extrait juste la partie après ":"
    if let Some(id_part) = id.split(':').nth(1) {
        id_part
    } else {
        id
    }
}

/// Crée un Thing pour un album
pub fn create_album_thing(album_id: &str) -> Thing {
    let clean_id = parse_id_part(album_id);
    Thing::from(("album".to_string(), clean_id.to_string()))
}

/// Crée un Thing pour un utilisateur
pub fn create_user_thing(user_id: &str) -> Thing {
    let clean_id = parse_id_part(user_id);
    Thing::from(("user".to_string(), clean_id.to_string()))
}

/// Crée un Thing pour une chanson
pub fn create_song_thing(song_id: &str) -> Thing {
    let clean_id = parse_id_part(song_id);
    Thing::from(("song".to_string(), clean_id.to_string()))
}

/// Crée un Thing pour un artiste
pub fn create_artist_thing(artist_id: &str) -> Thing {
    let clean_id = parse_id_part(artist_id);
    Thing::from(("artist".to_string(), clean_id.to_string()))
}

/// Crée un Thing pour un artiste
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
