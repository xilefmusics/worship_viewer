-- Align `song` with the repo schema for SurrealDB 2.6.x: normalize 3.x exports
-- (`FULLTEXT ANALYZER`, `BM25(k,b)`, parenthesized ASSERT, `($before).id`,
-- typed closure params) onto the same definitions as `20240301000008_song.surql`,
-- `20260410140000_song_chordlib08_drop_scalar_metadata.surql`,
-- `20260413160000_resource_owner_rename_from_teamowner.surql`, and
-- `20260418130000_surrealql_idiom_v3_compat.surql`.
--
-- Requires prior migrations that define `fn::song_normalize_metadata_array`,
-- `fn::song_nonempty_strs`, `fn::song_search_content_from_data`,
-- `fn::song_link_array_without_song`, and analyzer `text_search`.
--
-- Drop legacy `teamowner` document keys before schema fields: `REMOVE FIELD` alone
-- does not delete stored values (SurrealDB docs).

UPDATE song UNSET teamowner;
REMOVE FIELD IF EXISTS teamowner ON song;

DEFINE FIELD OVERWRITE blobs ON song TYPE array<record<blob>>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE blobs.* ON song TYPE record<blob>;

DEFINE FIELD OVERWRITE data ON song TYPE object
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE data.artists ON song TYPE array<string>
    DEFAULT ALWAYS []
    VALUE fn::song_normalize_metadata_array($value, $before)
    ASSERT array::len($value) > 0;

DEFINE FIELD OVERWRITE data.artists.* ON song TYPE string;

DEFINE FIELD OVERWRITE data.copyright ON song TYPE option<string>;

DEFINE FIELD OVERWRITE data.key ON song TYPE option<object>;

DEFINE FIELD OVERWRITE data.key.level ON song TYPE int
    ASSERT $value >= 0 AND $value < 12;

DEFINE FIELD OVERWRITE data.languages ON song TYPE array<string>
    DEFAULT ALWAYS []
    VALUE fn::song_normalize_metadata_array($value, $before)
    ASSERT array::len($value) > 0;

DEFINE FIELD OVERWRITE data.languages.* ON song TYPE string;

DEFINE FIELD OVERWRITE data.sections ON song TYPE array<object>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE data.sections.* ON song TYPE object;

DEFINE FIELD OVERWRITE data.sections.*.lines ON song TYPE array<object>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE data.sections.*.lines.* ON song TYPE object;

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts ON song TYPE array<object>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.* ON song TYPE object;

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord ON song TYPE option<object>;

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.base ON song TYPE option<object>;

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.base.level ON song TYPE int
    ASSERT $value >= 0 AND $value < 12;

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.duration ON song TYPE option<int>
    ASSERT $value = NONE OR $value >= 0;

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.kind ON song TYPE string
    ASSERT $value INSIDE ["Major", "Minor", "Diminished", "Augmented", "Suspended2", "Suspended4"];

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.main ON song TYPE object;

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.main.level ON song TYPE int
    ASSERT $value >= 0 AND $value < 12;

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.optional ON song TYPE bool
    VALUE $value ?? $before ?? false;

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.chord.var ON song TYPE string
    VALUE $value ?? $before ?? "";

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.comment ON song TYPE bool
    VALUE $value ?? $before ?? false;

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.languages ON song TYPE array<string>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE data.sections.*.lines.*.parts.*.languages.* ON song TYPE string;

DEFINE FIELD OVERWRITE data.sections.*.repeat_count ON song TYPE int
    VALUE $value ?? $before ?? 1
    ASSERT $value >= 1;

DEFINE FIELD OVERWRITE data.sections.*.title ON song TYPE string;

DEFINE FIELD OVERWRITE data.subtitle ON song TYPE option<string>;

DEFINE FIELD OVERWRITE data.tags ON song TYPE option<object>;

DEFINE FIELD OVERWRITE data.tags.* ON song TYPE string;

DEFINE FIELD OVERWRITE data.tempo ON song TYPE option<int>
    ASSERT $value = NONE OR $value >= 0;

DEFINE FIELD OVERWRITE data.time ON song TYPE option<array<int>>
    ASSERT $value = NONE OR array::len($value) = 2;

DEFINE FIELD OVERWRITE data.time.* ON song TYPE int;

DEFINE FIELD OVERWRITE data.titles ON song TYPE array<string>
    DEFAULT ALWAYS []
    VALUE fn::song_normalize_metadata_array($value, $before)
    ASSERT array::len($value) > 0;

DEFINE FIELD OVERWRITE data.titles.* ON song TYPE string;

DEFINE FIELD OVERWRITE not_a_song ON song TYPE bool
    VALUE $value ?? $before ?? false;

DEFINE FIELD OVERWRITE owner ON song TYPE record<team>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE search_content ON song TYPE string
    DEFAULT ''
    VALUE fn::song_search_content_from_data(data);

DEFINE INDEX OVERWRITE song_content_search_idx ON song
    FIELDS search_content SEARCH ANALYZER text_search BM25;

DEFINE INDEX OVERWRITE song_data_artists_search_idx ON song
    FIELDS data.artists SEARCH ANALYZER text_search BM25;

DEFINE INDEX OVERWRITE song_data_titles_search_idx ON song
    FIELDS data.titles SEARCH ANALYZER text_search BM25;

DEFINE EVENT OVERWRITE collection_song_remove ON song
    WHEN $event = "DELETE"
    THEN (
        UPDATE collection
            SET songs = fn::song_link_array_without_song(songs, $before.id)
            WHERE $before.id INSIDE array::map(songs, |$e| $e.id)
    );

DEFINE EVENT OVERWRITE like_song_cascade ON song
    WHEN $event = "DELETE"
    THEN DELETE like WHERE song = $before.id;

DEFINE EVENT OVERWRITE setlist_song_remove ON song
    WHEN $event = "DELETE"
    THEN (
        UPDATE setlist
            SET songs = fn::song_link_array_without_song(songs, $before.id)
            WHERE $before.id INSIDE array::map(songs, |$e| $e.id)
    );

UPDATE song SET search_content = fn::song_search_content_from_data(data);
