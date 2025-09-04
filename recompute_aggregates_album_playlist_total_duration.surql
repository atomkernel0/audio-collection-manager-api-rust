-- Recompute aggregates for all ALBUM records (scalar-safe; handles legacy reverse edges)
FOR $aid IN (SELECT VALUE id FROM album) {
  -- Counts via graph traversal (canonical direction: in = album, out = song)
  LET $tracks_edges = (
    SELECT count() FROM $aid->album_contains_song->song GROUP ALL
  )[0].count OR 0;

  -- Fallback count via legacy reversed traversal (out = album, in = song)
  LET $tracks_rev = (
    SELECT count() FROM $aid<-album_contains_song<-song GROUP ALL
  )[0].count OR 0;

  LET $total_tracks = IF $tracks_edges > 0 { $tracks_edges } ELSE { $tracks_rev };

  -- Sum duration as nanos (alias to ensure scalar extraction)
  LET $ns_primary_row = (
    SELECT math::sum(duration::nanos(duration)) AS ns
    FROM $aid->album_contains_song->song
    GROUP ALL
  )[0];

  LET $ns_fallback_row = (
    SELECT math::sum(duration::nanos(duration)) AS ns
    FROM $aid<-album_contains_song<-song
    GROUP ALL
  )[0];

  LET $ns_primary  = $ns_primary_row.ns  OR 0;
  LET $ns_fallback = $ns_fallback_row.ns OR 0;

  LET $total_ns = IF $tracks_edges > 0 { $ns_primary } ELSE { $ns_fallback };
  LET $total_duration = duration::from::nanos($total_ns);

  UPDATE $aid SET total_tracks = $total_tracks, total_duration = $total_duration;
};

-- Recompute aggregates for all PLAYLIST records (scalar-safe)
FOR $pid IN (SELECT VALUE id FROM playlist) {
  -- Count via traversal (extract scalar with .count)
  LET $songs_count = (
    SELECT count() FROM $pid->playlist_contains_song->song GROUP ALL
  )[0].count OR 0;

  -- Sum duration as nanos with alias
  LET $ns_row = (
    SELECT math::sum(duration::nanos(duration)) AS ns
    FROM $pid->playlist_contains_song->song
    GROUP ALL
  )[0];
  LET $ns = $ns_row.ns OR 0;

  UPDATE $pid SET songs_count = $songs_count, total_duration = duration::from::nanos($ns);
};
