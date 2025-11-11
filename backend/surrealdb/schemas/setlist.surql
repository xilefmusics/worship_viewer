DEFINE TABLE OVERWRITE setlist SCHEMAFULL;

DEFINE FIELD OVERWRITE title ON setlist TYPE string
    ASSERT string::len(string::trim($value)) > 0;

DEFINE FIELD OVERWRITE songs ON setlist TYPE array<object>
    VALUE $value ?? $before ?? [];
