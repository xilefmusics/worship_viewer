DEFINE TABLE OVERWRITE blob TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE created_at ON blob TYPE datetime READONLY VALUE $before ?? time::now() PERMISSIONS FULL;
DEFINE FIELD OVERWRITE file_type ON blob TYPE string ASSERT $value INSIDE ['image/png', 'image/jpeg', 'image/svg', 'image/svg+xml'] PERMISSIONS FULL;
DEFINE FIELD OVERWRITE height ON blob TYPE int ASSERT $value >= 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE ocr ON blob TYPE string VALUE $value ?? $before ?? '' PERMISSIONS FULL;
DEFINE FIELD OVERWRITE owner ON blob TYPE record<team> ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE width ON blob TYPE int ASSERT $value >= 0 PERMISSIONS FULL;

DEFINE EVENT OVERWRITE collection_cover_blob_cascade ON blob WHEN $event = 'DELETE' THEN (DELETE collection WHERE cover = $before.id);
DEFINE EVENT OVERWRITE song_blob_remove ON blob WHEN $event = 'DELETE' THEN (UPDATE song SET blobs = fn::blob_array_without_id(blobs, $before.id) WHERE $before.id INSIDE blobs);
