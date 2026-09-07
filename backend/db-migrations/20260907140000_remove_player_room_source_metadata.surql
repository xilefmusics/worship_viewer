-- Room content is captured at creation time; source metadata is not durable room state.
UPDATE player_room UNSET source_type;
UPDATE player_room UNSET source_id;
UPDATE player_room UNSET source_title;

REMOVE FIELD IF EXISTS source_type ON player_room;
REMOVE FIELD IF EXISTS source_id ON player_room;
REMOVE FIELD IF EXISTS source_title ON player_room;
