-- Align `setlist` with the repo schema for SurrealDB 2.6.x: `record<team>` owner,
-- nested `songs` entries, and full-text index using `SEARCH ANALYZER text_search BM25`
-- (`20240301000007_setlist.surql`, `20260413160000_resource_owner_rename_from_teamowner.surql`,
-- `20260328000002_fulltext_search.surql`). Normalizes 3.x exports (`FULLTEXT`,
-- `BM25(k,b)`, parenthesized ASSERT).
--
-- Legacy `teamowner`: clear document data, then remove field definition if present.

UPDATE setlist UNSET teamowner;
REMOVE FIELD IF EXISTS teamowner ON setlist;

DEFINE FIELD OVERWRITE owner ON setlist TYPE record<team>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE title ON setlist TYPE string
    ASSERT string::len(string::trim($value)) > 0;

DEFINE FIELD OVERWRITE songs ON setlist TYPE array<object>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE songs.* ON setlist TYPE object;

DEFINE FIELD OVERWRITE songs.*.id ON setlist TYPE record<song>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE songs.*.nr ON setlist TYPE option<string>;

DEFINE FIELD OVERWRITE songs.*.key ON setlist TYPE option<object>;

DEFINE FIELD OVERWRITE songs.*.key.level ON setlist TYPE int
    ASSERT $value >= 0 AND $value < 12;

DEFINE INDEX OVERWRITE setlist_title_search_idx ON setlist
    FIELDS title SEARCH ANALYZER text_search BM25;
