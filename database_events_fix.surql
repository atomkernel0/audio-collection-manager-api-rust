-- Fix pour l'événement update_user_aggregated_stats
-- Le problème est que time::unix() attend une datetime, pas une duration

-- Supprimer l'ancien événement
REMOVE EVENT update_user_aggregated_stats ON TABLE user_listens_song;

-- Recréer l'événement avec la correction
DEFINE EVENT update_user_aggregated_stats ON TABLE user_listens_song WHEN $event = "CREATE" OR $event = "UPDATE" THEN {
    LET $user_id = IF $event = 'CREATE' { $after.in } ELSE { $after.in };
    
    -- Mettre à jour le compteur d'écoutes de l'utilisateur (somme de toutes les écoutes agrégées)
    LET $user_listen_count = (SELECT math::sum(total_listens) AS total FROM user_listens_song WHERE in = $user_id GROUP ALL)[0].total OR 0;
    
    -- Calculer le temps total d'écoute (somme de toutes les durées agrégées)
    LET $total_listening_duration = (
        SELECT math::sum(total_duration) AS total_duration
        FROM user_listens_song
        WHERE in = $user_id
        GROUP ALL
    )[0].total_duration OR 0s;
    
    -- Convertir la durée en secondes (utiliser duration::secs au lieu de time::unix)
    LET $total_listening_time = math::floor(duration::secs($total_listening_duration));
    
    UPDATE $user_id SET
        listen_count = $user_listen_count,
        total_listening_time = $total_listening_time;
};