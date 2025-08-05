-- #################
-- # MIGRATION SCRIPT - Database Events
-- # Ce script applique ou met à jour les événements de base de données
-- #################

-- Supprimer les événements existants s'ils existent déjà (pour éviter les conflits)
REMOVE EVENT IF EXISTS update_album_aggregates ON TABLE album_contains_song;
REMOVE EVENT IF EXISTS update_artist_likes_count ON TABLE user_likes_artist;
REMOVE EVENT IF EXISTS update_user_favorite_count ON TABLE user_likes_album;
REMOVE EVENT IF EXISTS update_user_favorite_count_song ON TABLE user_likes_song;
REMOVE EVENT IF EXISTS update_user_favorite_count_artist ON TABLE user_likes_artist;
REMOVE EVENT IF EXISTS update_user_favorite_count_playlist ON TABLE user_likes_playlist;
REMOVE EVENT IF EXISTS update_playlist_stats ON TABLE playlist_contains_song;
REMOVE EVENT IF EXISTS update_playlist_likes_count ON TABLE user_likes_playlist;

-- #################
-- # Events - Mise à jour des agrégats d'albums
-- #################
DEFINE EVENT update_album_aggregates ON TABLE album_contains_song WHEN $event = "CREATE" OR $event = "DELETE" THEN {
    LET $album_id = if $event = "CREATE" { $after.in } else { $before.in };
    
    LET $total_tracks = (SELECT count() FROM album_contains_song WHERE in = $album_id GROUP ALL)[0].count;
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

    LET $total_likes = (SELECT count() FROM user_likes_artist WHERE out = $artist_id GROUP ALL)[0].count OR 0;
    UPDATE $artist_id SET total_likes = $total_likes;
};

-- Événement pour mettre à jour le compteur de favoris de l'utilisateur (albums)
DEFINE EVENT update_user_favorite_count ON TABLE user_likes_album WHEN $event = "CREATE" OR $event = "DELETE" THEN {
    LET $user_id = if $event = "CREATE" { $after.in } else { $before.in };

    LET $album_favorites = (SELECT count() FROM user_likes_album WHERE in = $user_id GROUP ALL)[0].count OR 0;
    LET $song_favorites = (SELECT count() FROM user_likes_song WHERE in = $user_id GROUP ALL)[0].count OR 0;
    LET $artist_favorites = (SELECT count() FROM user_likes_artist WHERE in = $user_id GROUP ALL)[0].count OR 0;

    UPDATE $user_id SET favorite_count = $album_favorites + $song_favorites + $artist_favorites;
};

-- Événement pour mettre à jour le compteur de favoris de l'utilisateur (songs)
DEFINE EVENT update_user_favorite_count_song ON TABLE user_likes_song WHEN $event = "CREATE" OR $event = "DELETE" THEN {
    LET $user_id = if $event = "CREATE" { $after.in } else { $before.in };

    LET $album_favorites = (SELECT count() FROM user_likes_album WHERE in = $user_id GROUP ALL)[0].count OR 0;
    LET $song_favorites = (SELECT count() FROM user_likes_song WHERE in = $user_id GROUP ALL)[0].count OR 0;
    LET $artist_favorites = (SELECT count() FROM user_likes_artist WHERE in = $user_id GROUP ALL)[0].count OR 0;

    UPDATE $user_id SET favorite_count = $album_favorites + $song_favorites + $artist_favorites;
};

-- Événement pour mettre à jour le compteur de favoris de l'utilisateur (artists)
DEFINE EVENT update_user_favorite_count_artist ON TABLE user_likes_artist WHEN $event = "CREATE" OR $event = "DELETE" THEN {
    LET $user_id = if $event = "CREATE" { $after.in } else { $before.in };

    LET $album_favorites = (SELECT count() FROM user_likes_album WHERE in = $user_id GROUP ALL)[0].count OR 0;
    LET $song_favorites = (SELECT count() FROM user_likes_song WHERE in = $user_id GROUP ALL)[0].count OR 0;
    LET $artist_favorites = (SELECT count() FROM user_likes_artist WHERE in = $user_id GROUP ALL)[0].count OR 0;
    LET $playlist_favorites = (SELECT count() FROM user_likes_playlist WHERE in = $user_id GROUP ALL)[0].count OR 0;

    UPDATE $user_id SET favorite_count = $album_favorites + $song_favorites + $artist_favorites + $playlist_favorites;
};

-- Événement pour mettre à jour le compteur de favoris de l'utilisateur (playlists)
DEFINE EVENT update_user_favorite_count_playlist ON TABLE user_likes_playlist WHEN $event = "CREATE" OR $event = "DELETE" THEN {
    LET $user_id = if $event = "CREATE" { $after.in } else { $before.in };

    LET $album_favorites = (SELECT count() FROM user_likes_album WHERE in = $user_id GROUP ALL)[0].count OR 0;
    LET $song_favorites = (SELECT count() FROM user_likes_song WHERE in = $user_id GROUP ALL)[0].count OR 0;
    LET $artist_favorites = (SELECT count() FROM user_likes_artist WHERE in = $user_id GROUP ALL)[0].count OR 0;
    LET $playlist_favorites = (SELECT count() FROM user_likes_playlist WHERE in = $user_id GROUP ALL)[0].count OR 0;

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
    LET $playlist_id = IF $event = 'CREATE' { $after.out } ELSE { $before.out };
    LET $total_likes = (SELECT count() FROM user_likes_playlist WHERE out = $playlist_id GROUP ALL)[0].count OR 0;
    
    UPDATE $playlist_id SET total_likes = $total_likes;
};

-- #################
-- # RECALCUL DES COMPTEURS EXISTANTS
-- # Ces requêtes mettent à jour les compteurs pour les données existantes
-- #################

-- Recalculer les agrégats d'albums
FOR $album IN (SELECT id FROM album) {
    LET $total_tracks = (SELECT count() FROM album_contains_song WHERE in = $album.id GROUP ALL)[0].count OR 0;
    LET $total_duration = (SELECT math::sum(out.duration) AS total FROM album_contains_song WHERE in = $album.id GROUP ALL)[0].total OR 0s;
    UPDATE $album.id SET total_tracks = $total_tracks, total_duration = $total_duration;
};

-- Recalculer les compteurs de likes d'artistes
FOR $artist IN (SELECT id FROM artist) {
    LET $total_likes = (SELECT count() FROM user_likes_artist WHERE out = $artist.id GROUP ALL)[0].count OR 0;
    UPDATE $artist.id SET total_likes = $total_likes;
};

-- Recalculer les compteurs de favoris des utilisateurs
FOR $user IN (SELECT id FROM user) {
    LET $album_favorites = (SELECT count() FROM user_likes_album WHERE in = $user.id GROUP ALL)[0].count OR 0;
    LET $song_favorites = (SELECT count() FROM user_likes_song WHERE in = $user.id GROUP ALL)[0].count OR 0;
    LET $artist_favorites = (SELECT count() FROM user_likes_artist WHERE in = $user.id GROUP ALL)[0].count OR 0;
    LET $playlist_favorites = (SELECT count() FROM user_likes_playlist WHERE in = $user.id GROUP ALL)[0].count OR 0;
    
    UPDATE $user.id SET favorite_count = $album_favorites + $song_favorites + $artist_favorites + $playlist_favorites;
};

-- Recalculer les statistiques des playlists
FOR $playlist IN (SELECT id FROM playlist) {
    LET $songs_count = (SELECT count() AS cnt FROM playlist_contains_song WHERE in = $playlist.id GROUP ALL)[0].cnt OR 0;
    LET $total_duration = (SELECT math::sum(out.duration) AS total FROM playlist_contains_song WHERE in = $playlist.id GROUP ALL)[0].total OR 0s;
    LET $total_likes = (SELECT count() FROM user_likes_playlist WHERE out = $playlist.id GROUP ALL)[0].count OR 0;
    
    UPDATE $playlist.id SET 
        songs_count = $songs_count, 
        total_duration = $total_duration,
        total_likes = $total_likes;
};

-- Afficher un message de confirmation
SELECT "Migration des événements terminée avec succès!" AS message;