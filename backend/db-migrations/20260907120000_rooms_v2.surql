-- Rooms v2: independent rooms, shared queue, join lock, and optional source metadata.
-- Existing v1 player_room rows keep their captured content.

DEFINE FIELD OVERWRITE source_type ON player_room TYPE none | string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE source_id ON player_room TYPE none | string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE source_title ON player_room TYPE none | string PERMISSIONS FULL;

-- Account identity owns the room; participant IDs remain connection-scoped authority.
DEFINE FIELD OVERWRITE host_user_id ON player_room TYPE none | string PERMISSIONS FULL;

DEFINE FIELD OVERWRITE queue_json ON player_room TYPE string DEFAULT "[]" PERMISSIONS FULL;
UPDATE player_room SET queue_json = "[]" WHERE queue_json = NONE;

DEFINE FIELD OVERWRITE queue_votes_json ON player_room TYPE string DEFAULT "{}" PERMISSIONS FULL;
UPDATE player_room SET queue_votes_json = "{}" WHERE queue_votes_json = NONE;

-- Existing rooms keep library add-access; new rooms set `open` in application code.
DEFINE FIELD OVERWRITE open ON player_room TYPE bool DEFAULT true PERMISSIONS FULL;
UPDATE player_room SET open = true WHERE open = NONE;

DEFINE FIELD OVERWRITE locked ON player_room TYPE bool DEFAULT false PERMISSIONS FULL;
UPDATE player_room SET locked = false WHERE locked = NONE;

REMOVE FIELD IF EXISTS song_pool_json ON player_room;
UPDATE player_room UNSET song_pool_json;

-- Preserve rooms whose host lease is still current, but classify rooms that
-- already expired before the lifetime rule changed so they are not resurrected.
UPDATE player_room
SET closed_at = time::now()
WHERE closed_at = NONE AND host_lease_expires_at <= time::now();

DEFINE INDEX OVERWRITE player_room_active_owner_idx ON player_room FIELDS owner, closed_at CONCURRENTLY;
