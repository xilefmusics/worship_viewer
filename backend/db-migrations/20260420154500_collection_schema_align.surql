-- Align `collection` with `20240301000001_collection.surql`, `20260328000002_fulltext_search.surql`,
-- and `20260413160000_resource_owner_rename_from_teamowner.surql` for SurrealDB 2.6.x.
-- Normalizes 3.x exports (`FULLTEXT`, `BM25(k,b)`, `FIELDS`-only index spelling is kept as
-- `FIELDS … SEARCH ANALYZER … BM25` like other repo migrations).
--
-- Legacy `teamowner`: clear document data, then remove field definition if present.

UPDATE collection UNSET teamowner;
REMOVE FIELD IF EXISTS teamowner ON collection;

DEFINE FIELD OVERWRITE owner ON collection TYPE record<team>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE title ON collection TYPE string
    ASSERT string::len(string::trim($value)) > 0;

DEFINE FIELD OVERWRITE cover ON collection TYPE option<record<blob>>;

DEFINE FIELD OVERWRITE songs ON collection TYPE array<object>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE songs.* ON collection TYPE object;

DEFINE FIELD OVERWRITE songs.*.id ON collection TYPE record<song>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE songs.*.nr ON collection TYPE option<string>;

DEFINE FIELD OVERWRITE songs.*.key ON collection TYPE option<object>;

DEFINE FIELD OVERWRITE songs.*.key.level ON collection TYPE int
    ASSERT $value >= 0 AND $value < 12;

DEFINE INDEX OVERWRITE collection_title_search_idx ON collection
    FIELDS title SEARCH ANALYZER text_search BM25;

DEFINE EVENT OVERWRITE collection_default_collection_unset ON collection
    WHEN $event = "DELETE"
    THEN (
        UPDATE user
            SET default_collection = NONE
            WHERE default_collection = $before.id
    );
