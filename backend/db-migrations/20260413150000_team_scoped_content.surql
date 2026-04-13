-- Drop legacy user ACL and resource owner; content is authorized via teamowner + team membership.

REMOVE EVENT IF EXISTS blob_user_cascade ON user;
REMOVE EVENT IF EXISTS collection_user_cascade ON user;
REMOVE EVENT IF EXISTS setlist_user_cascade ON user;
REMOVE EVENT IF EXISTS song_user_cascade ON user;

REMOVE FIELD IF EXISTS read ON user;
REMOVE FIELD IF EXISTS write ON user;

REMOVE FIELD IF EXISTS owner ON blob;
REMOVE FIELD IF EXISTS owner ON song;
REMOVE FIELD IF EXISTS owner ON collection;
REMOVE FIELD IF EXISTS owner ON setlist;

-- When a personal team is deleted (user cascade), remove content scoped to that team.
-- One statement per event (SurrealQL THEN does not accept a parenthesized multi-statement block).
DEFINE EVENT OVERWRITE team_personal_blob_cascade ON team
    WHEN $event = "DELETE" AND $before.owner IS NOT NONE
    THEN DELETE blob WHERE teamowner = $before.id;

DEFINE EVENT OVERWRITE team_personal_song_cascade ON team
    WHEN $event = "DELETE" AND $before.owner IS NOT NONE
    THEN DELETE song WHERE teamowner = $before.id;

DEFINE EVENT OVERWRITE team_personal_collection_cascade ON team
    WHEN $event = "DELETE" AND $before.owner IS NOT NONE
    THEN DELETE collection WHERE teamowner = $before.id;

DEFINE EVENT OVERWRITE team_personal_setlist_cascade ON team
    WHEN $event = "DELETE" AND $before.owner IS NOT NONE
    THEN DELETE setlist WHERE teamowner = $before.id;
