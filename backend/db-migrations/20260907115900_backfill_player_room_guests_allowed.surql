-- Rooms created before guests_allowed was introduced may not have the field.
-- Backfill them before later migrations write those records and trigger schema
-- validation for every field.
UPDATE player_room SET guests_allowed = true WHERE guests_allowed = NONE;
