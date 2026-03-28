DEFINE TABLE OVERWRITE setlist SCHEMAFULL;

DEFINE FIELD OVERWRITE owner ON setlist TYPE record<user>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE title ON setlist TYPE string
    ASSERT string::len(string::trim($value)) > 0;

DEFINE FIELD OVERWRITE songs ON setlist TYPE array<object>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE songs.*.id ON setlist TYPE record<song>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE songs.*.nr ON setlist TYPE option<string>;
DEFINE FIELD OVERWRITE songs.*.key ON setlist TYPE option<object>;
DEFINE FIELD OVERWRITE songs.*.key.level ON setlist TYPE int
    ASSERT $value >= 0 AND $value < 12;

DEFINE EVENT OVERWRITE setlist_user_cascade ON user
    WHEN $event = "DELETE"
    THEN DELETE setlist WHERE owner = $before.id;

DEFINE EVENT OVERWRITE setlist_song_remove ON song
    WHEN $event = "DELETE"
    THEN (
        UPDATE setlist
            SET songs = songs[WHERE $this.id != $before.id]
            WHERE $before.id INSIDE songs.*.id
    );
