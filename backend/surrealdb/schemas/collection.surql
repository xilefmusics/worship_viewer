DEFINE TABLE OVERWRITE collection SCHEMAFULL;

DEFINE FIELD OVERWRITE owner ON collection TYPE record<user>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE title ON collection TYPE string
    ASSERT string::len(string::trim($value)) > 0;

DEFINE FIELD OVERWRITE cover ON collection TYPE string
    VALUE $value ?? $before ?? "";

DEFINE FIELD OVERWRITE songs ON collection TYPE array<object>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE songs.*.id ON collection TYPE record<song>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE songs.*.nr ON collection TYPE option<string>;
DEFINE FIELD OVERWRITE songs.*.key ON collection TYPE option<object>;
DEFINE FIELD OVERWRITE songs.*.key.level ON collection TYPE int
    ASSERT $value >= 0 AND $value < 12;
