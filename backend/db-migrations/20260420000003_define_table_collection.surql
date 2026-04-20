DEFINE TABLE OVERWRITE collection TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE cover ON collection TYPE none | record<blob> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE owner ON collection TYPE record<team> ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE songs ON collection TYPE array<object> VALUE $value ?? $before ?? [] PERMISSIONS FULL;
DEFINE FIELD OVERWRITE songs.*.id ON collection TYPE record<song> ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE songs.*.key ON collection TYPE none | object PERMISSIONS FULL;
DEFINE FIELD OVERWRITE songs.*.key.level ON collection TYPE int ASSERT $value >= 0 AND $value < 12 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE songs.*.nr ON collection TYPE none | string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE title ON collection TYPE string ASSERT string::len(string::trim($value)) > 0 PERMISSIONS FULL;

DEFINE INDEX OVERWRITE collection_title_search_idx ON collection FIELDS title FULLTEXT ANALYZER text_search BM25(1.2,0.75) CONCURRENTLY;
DEFINE EVENT OVERWRITE collection_default_collection_unset ON collection WHEN $event = 'DELETE' THEN (UPDATE user SET default_collection = NONE WHERE default_collection = $before.id);
