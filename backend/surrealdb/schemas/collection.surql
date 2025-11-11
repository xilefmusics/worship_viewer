DEFINE TABLE OVERWRITE collection SCHEMAFULL;

DEFINE FIELD OVERWRITE title ON collection TYPE string
    ASSERT string::len(string::trim($value)) > 0;

DEFINE FIELD OVERWRITE cover ON collection TYPE string
    VALUE $value ?? $before ?? "";

DEFINE FIELD OVERWRITE songs ON collection TYPE array<object>
    VALUE $value ?? $before ?? [];
