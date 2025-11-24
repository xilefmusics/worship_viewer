DEFINE TABLE OVERWRITE song SCHEMAFULL;

DEFINE FIELD OVERWRITE owner ON song TYPE record<user>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE not_a_song ON song TYPE bool
    VALUE $value ?? $before ?? false;

DEFINE FIELD OVERWRITE blobs ON song TYPE array<record<blob>>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE data ON song TYPE object
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE data.title ON song TYPE string;

DEFINE FIELD OVERWRITE data.subtitle ON song TYPE option<string>;
DEFINE FIELD OVERWRITE data.copyright ON song TYPE option<string>;
DEFINE FIELD OVERWRITE data.artist ON song TYPE option<string>;
DEFINE FIELD OVERWRITE data.language ON song TYPE option<string>;

DEFINE FIELD OVERWRITE data.tempo ON song TYPE option<int>
    ASSERT $value = NONE OR $value >= 0;

DEFINE FIELD OVERWRITE data.time ON song TYPE option<array<int>>
    ASSERT $value = NONE OR array::len($value) = 2;

DEFINE FIELD OVERWRITE data.key ON song TYPE option<object>;
DEFINE FIELD OVERWRITE data.key.level ON song TYPE int
    ASSERT $value >= 0 AND $value < 12;

DEFINE FIELD OVERWRITE data.sections ON song TYPE array<object>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE data.sections.*.title ON song TYPE string;

DEFINE FIELD OVERWRITE data.sections.*.lines ON song TYPE array<object>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts ON song TYPE array<object>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.languages ON song TYPE array<string>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.comment ON song TYPE bool
    VALUE $value ?? $before ?? false;

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord ON song TYPE option<object>;

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.main ON song TYPE object;
DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.main.level ON song TYPE int
    ASSERT $value >= 0 AND $value < 12;

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.base ON song TYPE option<object>;
DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.base.level ON song TYPE int
    ASSERT $value >= 0 AND $value < 12;

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.kind ON song TYPE string
    ASSERT $value INSIDE ["Major", "Minor", "Diminished", "Augmented", "Suspended2", "Suspended4"];

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.var ON song TYPE string
    VALUE $value ?? $before ?? "";

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.duration ON song TYPE option<int>
    ASSERT $value = NONE OR $value >= 0;

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.optional ON song TYPE bool
    VALUE $value ?? $before ?? false;

DEFINE EVENT OVERWRITE song_user_cascade ON user
    WHEN $event = "DELETE"
    THEN DELETE song WHERE owner = $before.id;

DEFINE EVENT OVERWRITE song_blob_remove ON blob
WHEN $event = "DELETE"
THEN (
    UPDATE song
            SET blobs = blobs[WHERE $this != $before.id]
            WHERE $before.id INSIDE blobs
);