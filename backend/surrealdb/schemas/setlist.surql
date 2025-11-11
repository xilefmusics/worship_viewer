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
