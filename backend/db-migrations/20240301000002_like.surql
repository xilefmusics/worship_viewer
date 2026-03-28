DEFINE TABLE OVERWRITE like SCHEMAFULL;

DEFINE FIELD OVERWRITE owner ON like TYPE record<user>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE song ON like TYPE record<song>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE created_at ON like TYPE datetime
    DEFAULT time::now()
    VALUE $before ?? $value
    READONLY;

DEFINE INDEX OVERWRITE like_owner_song_unique ON like COLUMNS owner, song UNIQUE;

DEFINE EVENT OVERWRITE like_user_cascade ON user
    WHEN $event = "DELETE"
    THEN DELETE like WHERE owner = $before.id;

DEFINE EVENT OVERWRITE like_song_cascade ON song
    WHEN $event = "DELETE"
    THEN DELETE like WHERE song = $before.id;
