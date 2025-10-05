-- Create table for anonymous listen tracking with TTL
DEFINE TABLE anonymous_listen_log SCHEMAFULL;

-- Define fields
DEFINE FIELD ip_address ON anonymous_listen_log TYPE string ASSERT $value != NONE;
DEFINE FIELD song_id ON anonymous_listen_log TYPE string ASSERT $value != NONE;
DEFINE FIELD listened_at ON anonymous_listen_log TYPE datetime DEFAULT time::now();

-- Index for efficient querying by IP and timestamp
DEFINE INDEX idx_anonymous_ip_time ON anonymous_listen_log FIELDS ip_address, listened_at;

-- Optional: Event to automatically delete records older than 2 minutes (cleanup)
DEFINE EVENT cleanup_anonymous_logs ON TABLE anonymous_listen_log WHEN true THEN {
    DELETE FROM anonymous_listen_log WHERE listened_at < time::now() - 2m;
};