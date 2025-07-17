-- #################
-- # TABLE album
-- #################
DEFINE TABLE album SCHEMAFULL;
DEFINE FIELD title ON TABLE album TYPE string;
DEFINE FIELD cover_url ON TABLE album TYPE option<string>;
DEFINE FIELD release_year ON TABLE album TYPE option<int>;
DEFINE FIELD genres ON TABLE album TYPE array<string> DEFAULT [];
DEFINE FIELD langs ON TABLE album TYPE array<string> DEFAULT [];
DEFINE FIELD dominant_color ON TABLE album TYPE option<string>;
DEFINE FIELD total_tracks ON TABLE album TYPE int DEFAULT 0;
DEFINE FIELD total_duration ON TABLE album TYPE duration DEFAULT 0s;
DEFINE FIELD total_listens ON TABLE album TYPE int DEFAULT 0;
DEFINE FIELD total_user_listens ON TABLE album TYPE int DEFAULT 0;
DEFINE FIELD total_likes ON TABLE album TYPE int DEFAULT 0;

-- #################
-- # TABLE artist
-- #################
DEFINE TABLE artist SCHEMAFULL;
DEFINE FIELD name ON TABLE artist TYPE string;
-- Assuming MusicGenre enum serializes to string
DEFINE FIELD genres ON TABLE artist TYPE array<string> DEFAULT [];
DEFINE FIELD country_code ON TABLE artist TYPE string;
DEFINE FIELD artist_image ON TABLE artist TYPE option<string>;
DEFINE FIELD albums_count ON TABLE artist TYPE int DEFAULT 0;
DEFINE FIELD songs_count ON TABLE artist TYPE int DEFAULT 0;
DEFINE FIELD total_likes ON TABLE artist TYPE int DEFAULT 0;

-- #################
-- # TABLE song
-- #################
DEFINE TABLE song SCHEMAFULL;
DEFINE FIELD title ON TABLE song TYPE string;
DEFINE FIELD file_url ON TABLE song TYPE string;
DEFINE FIELD duration ON TABLE song TYPE duration;
DEFINE FIELD song_index ON TABLE song TYPE int;
DEFINE FIELD tempo ON TABLE song TYPE float DEFAULT 0;
DEFINE FIELD total_listens ON TABLE song TYPE int DEFAULT 0;
DEFINE FIELD total_user_listens ON TABLE song TYPE int DEFAULT 0;
DEFINE FIELD total_likes ON TABLE song TYPE int DEFAULT 0;

-- #################
-- # TABLE user
-- #################
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD username ON TABLE user TYPE string;
DEFINE FIELD password ON TABLE user TYPE string;
DEFINE FIELD created_at ON TABLE user TYPE datetime;
DEFINE FIELD listen_count ON TABLE user TYPE int DEFAULT 0;
-- u64 can be stored in int but care must be taken with very large values
DEFINE FIELD total_listening_time ON TABLE user TYPE int DEFAULT 0;
DEFINE FIELD favorite_count ON TABLE user TYPE int DEFAULT 0;
DEFINE FIELD listening_streak ON TABLE user TYPE int DEFAULT 0;
-- Badge is a string enum in Rust
DEFINE FIELD badges ON TABLE user TYPE array<string> DEFAULT [];
DEFINE FIELD level ON TABLE user TYPE int DEFAULT 0;
DEFINE FIELD experience_points ON TABLE user TYPE int DEFAULT 0;


-- #################################################
-- # Relation Tables (Defined with IN/OUT for graph integrity)
-- #################################################
DEFINE TABLE artist_creates_album;
DEFINE FIELD in ON artist_creates_album TYPE record(artist);
DEFINE FIELD out ON artist_creates_album TYPE record(album);
DEFINE FIELD role ON TABLE artist_creates_album TYPE string DEFAULT 'Primary';
DEFINE FIELD created_at ON TABLE artist_creates_album TYPE datetime DEFAULT time::now();

DEFINE TABLE album_contains_song;
DEFINE FIELD in ON album_contains_song TYPE record(album);
DEFINE FIELD out ON album_contains_song TYPE record(song);
DEFINE FIELD created_at ON TABLE album_contains_song TYPE datetime DEFAULT time::now();

DEFINE TABLE artist_performs_song;
DEFINE FIELD in ON artist_performs_song TYPE record(artist);
DEFINE FIELD out ON artist_performs_song TYPE record(song);
DEFINE FIELD role ON TABLE artist_performs_song TYPE string DEFAULT 'Primary';
DEFINE FIELD created_at ON TABLE artist_performs_song TYPE datetime DEFAULT time::now();

DEFINE TABLE user_likes_album;
DEFINE FIELD in ON user_likes_album TYPE record<user>;
DEFINE FIELD out ON user_likes_album TYPE record<album>;
DEFINE FIELD created_at ON TABLE user_likes_album TYPE datetime DEFAULT time::now();

DEFINE TABLE user_likes_song;
DEFINE FIELD in ON user_likes_song TYPE record<user>;
DEFINE FIELD out ON user_likes_song TYPE record<song>;
DEFINE FIELD created_at ON TABLE user_likes_song TYPE datetime DEFAULT time::now();

DEFINE TABLE user_likes_artist;
DEFINE FIELD in ON user_likes_artist TYPE record<user>;
DEFINE FIELD out ON user_likes_artist TYPE record<artist>;
DEFINE FIELD created_at ON TABLE user_likes_artist TYPE datetime DEFAULT time::now();

-- Playlist relations
DEFINE TABLE user_creates_playlist;
DEFINE FIELD in ON user_creates_playlist TYPE record<user>;
DEFINE FIELD out ON user_creates_playlist TYPE record<playlist>;
DEFINE FIELD created_at ON TABLE user_creates_playlist TYPE datetime DEFAULT time::now();

DEFINE TABLE playlist_contains_song;
DEFINE FIELD in ON playlist_contains_song TYPE record<playlist>;
DEFINE FIELD out ON playlist_contains_song TYPE record<song>;
DEFINE FIELD added_at ON TABLE playlist_contains_song TYPE datetime DEFAULT time::now();
DEFINE FIELD added_by ON TABLE playlist_contains_song TYPE record<user>;

DEFINE TABLE user_likes_playlist;
DEFINE FIELD in ON user_likes_playlist TYPE record<user>;
DEFINE FIELD out ON user_likes_playlist TYPE record<playlist>;
DEFINE FIELD created_at ON TABLE user_likes_playlist TYPE datetime DEFAULT time::now();

-- #################################################
-- # Extensions des métadonnées pour les favoris
-- #################################################

-- Extension des métadonnées pour user_likes_album
DEFINE FIELD tags ON TABLE user_likes_album TYPE array<string> DEFAULT [];
DEFINE FIELD notes ON TABLE user_likes_album TYPE option<string>;
DEFINE FIELD user_rating ON TABLE user_likes_album TYPE option<int>
    ASSERT $value == NONE OR ($value >= 1 AND $value <= 5);
DEFINE FIELD sort_order ON TABLE user_likes_album TYPE int DEFAULT 0;
DEFINE FIELD is_favorite ON TABLE user_likes_album TYPE bool DEFAULT true;
DEFINE FIELD last_accessed ON TABLE user_likes_album TYPE option<datetime>;

-- Extension des métadonnées pour user_likes_song
DEFINE FIELD tags ON TABLE user_likes_song TYPE array<string> DEFAULT [];
DEFINE FIELD notes ON TABLE user_likes_song TYPE option<string>;
DEFINE FIELD user_rating ON TABLE user_likes_song TYPE option<int>
    ASSERT $value == NONE OR ($value >= 1 AND $value <= 5);
DEFINE FIELD sort_order ON TABLE user_likes_song TYPE int DEFAULT 0;
DEFINE FIELD is_favorite ON TABLE user_likes_song TYPE bool DEFAULT false;
DEFINE FIELD last_accessed ON TABLE user_likes_song TYPE option<datetime>;

-- Extension des métadonnées pour user_likes_artist
DEFINE FIELD tags ON TABLE user_likes_artist TYPE array<string> DEFAULT [];
DEFINE FIELD notes ON TABLE user_likes_artist TYPE option<string>;
DEFINE FIELD user_rating ON TABLE user_likes_artist TYPE option<int>
    ASSERT $value == NONE OR ($value >= 1 AND $value <= 5);
DEFINE FIELD sort_order ON TABLE user_likes_artist TYPE int DEFAULT 0;
DEFINE FIELD is_favorite ON TABLE user_likes_artist TYPE bool DEFAULT false;
DEFINE FIELD last_accessed ON TABLE user_likes_artist TYPE option<datetime>;

-- Extension des métadonnées pour user_likes_playlist
DEFINE FIELD tags ON TABLE user_likes_playlist TYPE array<string> DEFAULT [];
DEFINE FIELD notes ON TABLE user_likes_playlist TYPE option<string>;
DEFINE FIELD user_rating ON TABLE user_likes_playlist TYPE option<int>
    ASSERT $value == NONE OR ($value >= 1 AND $value <= 5);
DEFINE FIELD sort_order ON TABLE user_likes_playlist TYPE int DEFAULT 0;
DEFINE FIELD is_favorite ON TABLE user_likes_playlist TYPE bool DEFAULT false;
DEFINE FIELD last_accessed ON TABLE user_likes_playlist TYPE option<datetime>;


-- #################
-- # TABLE playlist
-- #################
DEFINE TABLE playlist SCHEMAFULL;
DEFINE FIELD name ON TABLE playlist TYPE string;
DEFINE FIELD cover_url ON TABLE playlist TYPE option<string>;
DEFINE FIELD is_public ON TABLE playlist TYPE bool DEFAULT false;
DEFINE FIELD dominant_color ON TABLE playlist TYPE option<string>;
DEFINE FIELD created_by ON TABLE playlist TYPE record<user>;

-- Timestamps
DEFINE FIELD created_at ON TABLE playlist TYPE datetime;
DEFINE FIELD updated_at ON TABLE playlist TYPE datetime;

-- Stats
DEFINE FIELD songs_count ON TABLE playlist TYPE int DEFAULT 0;
DEFINE FIELD total_duration ON TABLE playlist TYPE duration DEFAULT 0s;
DEFINE FIELD total_listens ON TABLE playlist TYPE int DEFAULT 0;
DEFINE FIELD total_likes ON TABLE playlist TYPE int DEFAULT 0;

-- #################
-- # TABLE user_favorites_preferences
-- #################
DEFINE TABLE user_favorites_preferences SCHEMAFULL;
DEFINE FIELD user_id ON TABLE user_favorites_preferences TYPE record<user>;
DEFINE FIELD default_sort_by ON TABLE user_favorites_preferences TYPE string DEFAULT 'created_at'
    ASSERT $value IN ['created_at', 'user_rating', 'sort_order', 'last_accessed', 'title'];
DEFINE FIELD default_sort_order ON TABLE user_favorites_preferences TYPE string DEFAULT 'DESC'
    ASSERT $value IN ['ASC', 'DESC'];
DEFINE FIELD items_per_page ON TABLE user_favorites_preferences TYPE int DEFAULT 20;
DEFINE FIELD show_ratings ON TABLE user_favorites_preferences TYPE bool DEFAULT true;
DEFINE FIELD show_notes ON TABLE user_favorites_preferences TYPE bool DEFAULT true;
DEFINE FIELD show_tags ON TABLE user_favorites_preferences TYPE bool DEFAULT true;
DEFINE FIELD auto_add_to_favorites ON TABLE user_favorites_preferences TYPE bool DEFAULT false;
DEFINE FIELD export_format ON TABLE user_favorites_preferences TYPE string DEFAULT 'json'
    ASSERT $value IN ['json', 'csv', 'xml'];
DEFINE FIELD created_at ON TABLE user_favorites_preferences TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON TABLE user_favorites_preferences TYPE datetime DEFAULT time::now();


-- #################
-- # INDEX pour les favoris
-- #################

-- Index pour les requêtes de favoris par utilisateur avec tri par date
DEFINE INDEX idx_user_likes_album_user_created ON user_likes_album FIELDS in, created_at;
DEFINE INDEX idx_user_likes_song_user_created ON user_likes_song FIELDS in, created_at;
DEFINE INDEX idx_user_likes_artist_user_created ON user_likes_artist FIELDS in, created_at;

-- Index pour les requêtes de tri personnalisé
DEFINE INDEX idx_user_likes_album_user_sort ON user_likes_album FIELDS in, sort_order;
DEFINE INDEX idx_user_likes_song_user_sort ON user_likes_song FIELDS in, sort_order;
DEFINE INDEX idx_user_likes_artist_user_sort ON user_likes_artist FIELDS in, sort_order;

-- Index pour les favoris marqués
DEFINE INDEX idx_user_likes_album_favorites ON user_likes_album FIELDS in, is_favorite;
DEFINE INDEX idx_user_likes_song_favorites ON user_likes_song FIELDS in, is_favorite;
DEFINE INDEX idx_user_likes_artist_favorites ON user_likes_artist FIELDS in, is_favorite;

-- Index pour les recherches par tags
DEFINE INDEX idx_user_likes_album_tags ON user_likes_album FIELDS in, tags;
DEFINE INDEX idx_user_likes_song_tags ON user_likes_song FIELDS in, tags;
DEFINE INDEX idx_user_likes_artist_tags ON user_likes_artist FIELDS in, tags;

-- Index unique pour les préférences utilisateur
DEFINE INDEX idx_user_favorites_preferences_unique ON user_favorites_preferences FIELDS user_id UNIQUE;

-- Index pour les requêtes de rating
DEFINE INDEX idx_user_likes_album_rating ON user_likes_album FIELDS in, user_rating;
DEFINE INDEX idx_user_likes_song_rating ON user_likes_song FIELDS in, user_rating;
DEFINE INDEX idx_user_likes_artist_rating ON user_likes_artist FIELDS in, user_rating;

-- Index pour les playlists
DEFINE INDEX idx_user_likes_playlist_user_created ON user_likes_playlist FIELDS in, created_at;
DEFINE INDEX idx_user_likes_playlist_user_sort ON user_likes_playlist FIELDS in, sort_order;
DEFINE INDEX idx_user_likes_playlist_favorites ON user_likes_playlist FIELDS in, is_favorite;
DEFINE INDEX idx_user_likes_playlist_tags ON user_likes_playlist FIELDS in, tags;
DEFINE INDEX idx_user_likes_playlist_rating ON user_likes_playlist FIELDS in, user_rating;
DEFINE INDEX idx_user_creates_playlist_user ON user_creates_playlist FIELDS in;
DEFINE INDEX idx_playlist_contains_song_playlist ON playlist_contains_song FIELDS in;
DEFINE INDEX idx_playlist_contains_song_song ON playlist_contains_song FIELDS out;
DEFINE INDEX idx_playlist_created_by ON playlist FIELDS created_by;


-- #################
-- # Events
-- #################
DEFINE EVENT update_album_aggregates ON TABLE album_contains_song WHEN $event = "CREATE" OR $event = "DELETE" THEN {
    LET $album_id = if $event = "CREATE" { $after.in } else { $before.in };
    
    LET $total_tracks = (SELECT count() FROM album_contains_song WHERE in = $album_id GROUP ALL).count;
    UPDATE $album_id SET total_tracks = $total_tracks;

    LET $total_duration = (SELECT math::sum(out.duration) AS total FROM album_contains_song WHERE in = $album_id GROUP ALL).total OR 0s;
    UPDATE $album_id SET total_duration = $total_duration;
};

-- #################
-- # EVENTS pour les compteurs de likes d'artistes
-- #################

-- Événement pour maintenir le compteur total_likes des artistes
DEFINE EVENT update_artist_likes_count ON TABLE user_likes_artist WHEN $event = "CREATE" OR $event = "DELETE" THEN {
    LET $artist_id = if $event = "CREATE" { $after.out } else { $before.out };

    LET $total_likes = (SELECT count() FROM user_likes_artist WHERE out = $artist_id GROUP ALL).count OR 0;
    UPDATE $artist_id SET total_likes = $total_likes;
};

-- Événement pour mettre à jour le compteur de favoris de l'utilisateur (albums)
DEFINE EVENT update_user_favorite_count ON TABLE user_likes_album WHEN $event = "CREATE" OR $event = "DELETE" THEN {
    LET $user_id = if $event = "CREATE" { $after.in } else { $before.in };

    LET $album_favorites = (SELECT count() FROM user_likes_album WHERE in = $user_id GROUP ALL).count OR 0;
    LET $song_favorites = (SELECT count() FROM user_likes_song WHERE in = $user_id GROUP ALL).count OR 0;
    LET $artist_favorites = (SELECT count() FROM user_likes_artist WHERE in = $user_id GROUP ALL).count OR 0;

    UPDATE $user_id SET favorite_count = $album_favorites + $song_favorites + $artist_favorites;
};

-- Événement pour mettre à jour le compteur de favoris de l'utilisateur (songs)
DEFINE EVENT update_user_favorite_count_song ON TABLE user_likes_song WHEN $event = "CREATE" OR $event = "DELETE" THEN {
    LET $user_id = if $event = "CREATE" { $after.in } else { $before.in };

    LET $album_favorites = (SELECT count() FROM user_likes_album WHERE in = $user_id GROUP ALL).count OR 0;
    LET $song_favorites = (SELECT count() FROM user_likes_song WHERE in = $user_id GROUP ALL).count OR 0;
    LET $artist_favorites = (SELECT count() FROM user_likes_artist WHERE in = $user_id GROUP ALL).count OR 0;

    UPDATE $user_id SET favorite_count = $album_favorites + $song_favorites + $artist_favorites;
};

-- Événement pour mettre à jour le compteur de favoris de l'utilisateur (artists)
DEFINE EVENT update_user_favorite_count_artist ON TABLE user_likes_artist WHEN $event = "CREATE" OR $event = "DELETE" THEN {
    LET $user_id = if $event = "CREATE" { $after.in } else { $before.in };

    LET $album_favorites = (SELECT count() FROM user_likes_album WHERE in = $user_id GROUP ALL).count OR 0;
    LET $song_favorites = (SELECT count() FROM user_likes_song WHERE in = $user_id GROUP ALL).count OR 0;
    LET $artist_favorites = (SELECT count() FROM user_likes_artist WHERE in = $user_id GROUP ALL).count OR 0;
    LET $playlist_favorites = (SELECT count() FROM user_likes_playlist WHERE in = $user_id GROUP ALL).count OR 0;

    UPDATE $user_id SET favorite_count = $album_favorites + $song_favorites + $artist_favorites + $playlist_favorites;
};

-- Événement pour mettre à jour le compteur de favoris de l'utilisateur (playlists)
DEFINE EVENT update_user_favorite_count_playlist ON TABLE user_likes_playlist WHEN $event = "CREATE" OR $event = "DELETE" THEN {
    LET $user_id = if $event = "CREATE" { $after.in } else { $before.in };

    LET $album_favorites = (SELECT count() FROM user_likes_album WHERE in = $user_id GROUP ALL).count OR 0;
    LET $song_favorites = (SELECT count() FROM user_likes_song WHERE in = $user_id GROUP ALL).count OR 0;
    LET $artist_favorites = (SELECT count() FROM user_likes_artist WHERE in = $user_id GROUP ALL).count OR 0;
    LET $playlist_favorites = (SELECT count() FROM user_likes_playlist WHERE in = $user_id GROUP ALL).count OR 0;

    UPDATE $user_id SET favorite_count = $album_favorites + $song_favorites + $artist_favorites + $playlist_favorites;
};

-- Événement pour mettre à jour les statistiques de la playlist
DEFINE EVENT update_playlist_stats ON TABLE playlist_contains_song WHEN $event = "CREATE" OR $event = "DELETE" THEN {
    LET $playlist_id = if $event = "CREATE" { $after.in } else { $before.in };
    
    -- Utiliser une sous-requête qui retourne directement la valeur scalaire
    LET $songs_count = (SELECT count() AS cnt FROM playlist_contains_song WHERE in = $playlist_id GROUP ALL)[0].cnt OR 0;
    LET $total_duration = (SELECT math::sum(out.duration) AS total FROM playlist_contains_song WHERE in = $playlist_id GROUP ALL)[0].total OR 0s;
    
    UPDATE $playlist_id SET songs_count = $songs_count, total_duration = $total_duration;
};

-- Événement pour maintenir le compteur total_likes des playlists
DEFINE EVENT update_playlist_likes_count ON TABLE user_likes_playlist WHEN $event = "CREATE" OR $event = "DELETE" THEN {
    LET $playlist_id = if $event = "CREATE" { $after.out } else { $before.out };

    LET $total_likes = (SELECT count() FROM user_likes_playlist WHERE out = $playlist_id GROUP ALL).count OR 0;
    UPDATE $playlist_id SET total_likes = $total_likes;
};
