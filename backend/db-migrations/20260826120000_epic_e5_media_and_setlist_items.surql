-- Epic E5 ships as one forward migration. Media tables are new; only existing
-- setlist song slots require data conversion.

DEFINE TABLE OVERWRITE media TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD OVERWRITE owner ON media TYPE record<team> ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE title ON media TYPE string ASSERT string::len(string::trim($value)) > 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE content_json ON media TYPE string ASSERT string::len($value) > 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE pending_revision_json ON media TYPE none | string PERMISSIONS FULL;
DEFINE INDEX OVERWRITE media_owner_idx ON media FIELDS owner CONCURRENTLY;
DEFINE INDEX OVERWRITE media_title_search_idx ON media FIELDS title FULLTEXT ANALYZER text_search BM25(1.2,0.75) CONCURRENTLY;

DEFINE TABLE OVERWRITE media_asset TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD OVERWRITE owner ON media_asset TYPE record<team> ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE media_id ON media_asset TYPE record<media> ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE kind ON media_asset TYPE string ASSERT $value IN ['video', 'audio', 'image', 'pdf', 'svg'] PERMISSIONS FULL;
DEFINE FIELD OVERWRITE content_type ON media_asset TYPE string ASSERT string::len($value) > 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE byte_length ON media_asset TYPE int ASSERT $value >= 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE status ON media_asset TYPE string ASSERT $value IN ['staging', 'final'] PERMISSIONS FULL;
DEFINE FIELD OVERWRITE operation_id ON media_asset TYPE none | string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE etag ON media_asset TYPE none | string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE created_at ON media_asset TYPE datetime READONLY VALUE $before ?? time::now() PERMISSIONS FULL;
DEFINE INDEX OVERWRITE media_asset_owner_idx ON media_asset FIELDS owner CONCURRENTLY;
DEFINE INDEX OVERWRITE media_asset_media_idx ON media_asset FIELDS media_id CONCURRENTLY;
DEFINE INDEX OVERWRITE media_asset_operation_idx ON media_asset FIELDS operation_id CONCURRENTLY;

-- Keep the media row and its owned asset rows in the same ownership domain.
-- Events execute as part of the mutation that triggered them, so a move is atomic.
DEFINE EVENT OVERWRITE media_owner_update ON media WHEN $event = 'UPDATE' AND $before.owner != $after.owner THEN (
  UPDATE media_asset SET owner = $after.owner WHERE media_id = $after.id
);

-- Replace canonical setlist `songs` with tagged ordered `items`. DB objects use
-- `kind` because SurrealQL reserves `type`; the API tag remains `type`.
DEFINE FIELD OVERWRITE items ON setlist TYPE array<object> VALUE $value ?? $before ?? [] PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.kind ON setlist TYPE string ASSERT $value IN ['song', 'media'] PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.id ON setlist TYPE record<song> | record<media> ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.key ON setlist TYPE none | object PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.key.level ON setlist TYPE none | int ASSERT $value = NONE OR ($value >= 0 AND $value < 12) PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.nr ON setlist TYPE none | string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.tempo ON setlist TYPE none | int ASSERT $value = NONE OR $value >= 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.language ON setlist TYPE none | string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.flow ON setlist TYPE none | array<object> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.flow.*.title ON setlist TYPE string ASSERT string::len(string::trim($value)) > 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.flow.*.occurrence_index ON setlist TYPE none | int ASSERT $value = NONE OR $value >= 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.flow.*.repeats ON setlist TYPE none | int ASSERT $value = NONE OR $value >= 1 PERMISSIONS FULL;

UPDATE setlist SET items = array::map(songs ?? [], |$s: any| {
    kind: 'song',
    id: $s.id,
    nr: $s.nr,
    key: $s.key,
    tempo: $s.tempo,
    language: $s.language,
    flow: $s.flow
});
REMOVE FIELD songs ON setlist;

DEFINE FUNCTION OVERWRITE fn::setlist_items_without_song($items: array, $song_id: any) {
  RETURN array::filter($items, |$e: any| $e.kind != 'song' OR $e.id != $song_id);
} PERMISSIONS FULL;
DEFINE EVENT OVERWRITE setlist_song_remove ON song WHEN $event = 'DELETE' THEN (
  UPDATE setlist SET items = fn::setlist_items_without_song(items, $before.id)
  WHERE $before.id INSIDE array::map(
    array::filter(items ?? [], |$e: any| $e.kind = 'song'),
    |$e: any| ($e.id)
  )
);
