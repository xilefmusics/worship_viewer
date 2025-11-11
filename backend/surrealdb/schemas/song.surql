DEFINE TABLE OVERWRITE song SCHEMAFULL;

DEFINE FIELD OVERWRITE not_a_song ON song TYPE bool
    VALUE $value ?? $before ?? false;

DEFINE FIELD OVERWRITE blobs ON song TYPE array<string>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE data ON song TYPE object
    ASSERT $value != NONE;
