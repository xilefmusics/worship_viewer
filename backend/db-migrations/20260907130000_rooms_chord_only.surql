-- Rooms contain only captured chord songs; room-scoped blob/media delivery is no longer needed.
-- Allow the legacy array field to be unset before removing its schema definition.
DEFINE FIELD OVERWRITE media_ids ON player_room TYPE none | array<string> PERMISSIONS FULL;

UPDATE player_room UNSET media_ids;
UPDATE player_room UNSET snapshot_json;
UPDATE player_room UNSET state_json;

REMOVE FIELD IF EXISTS media_ids ON player_room;
REMOVE FIELD IF EXISTS snapshot_json ON player_room;
REMOVE FIELD IF EXISTS state_json ON player_room;
